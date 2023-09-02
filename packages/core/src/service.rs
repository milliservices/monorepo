use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

use crate::store::{HostChannel, ModuleChannel, RecvMsg, SendMsg, ServiceStore};

#[derive(Debug, Clone)]
pub struct ModuleConfig {
  pub path: String,
  pub symbol: String,
  pub name: String,
}

pub struct ServiceModule {
  pub config: ModuleConfig,
  engine: Engine,
  instance_pre: InstancePre<ServiceStore>,
  pub module_channel: ModuleChannel,
  pub host_channel: HostChannel,
  // linker: Linker<ServiceStore>,
  // module: Module,
}

impl ServiceModule {
  pub async fn new(cfg: &ModuleConfig) -> Result<Self> {
    let module_config = cfg.clone();

    let mut engine_config = Config::default();
    engine_config.async_support(true);
    let engine = Engine::new(&engine_config)?;
    let module = Module::from_file(&engine, &module_config.path)?;

    let mut linker: Linker<ServiceStore> = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |store: &mut ServiceStore| &mut store.wasi_ctx)?;

    let module_name = "env";
    linker.func_wrap(module_name, "send_response", Self::cb_send_response)?;
    linker.func_wrap(module_name, "get_metadata", Self::cb_get_metadata)?;
    linker.func_wrap(
      module_name,
      "set_response_metadata",
      Self::cb_set_response_metadata,
    )?;
    linker.func_wrap2_async(
      module_name,
      "call_service",
      |caller, name_ptr: i32, data_ptr: i32| {
        Box::new(Self::cb_call_service(caller, name_ptr, data_ptr))
      },
    )?;

    let instance_pre = linker.instantiate_pre(&module)?;

    let (send_1, recv_1) = tokio::sync::mpsc::channel::<SendMsg>(10);
    let (send_2, recv_2) = tokio::sync::mpsc::channel::<RecvMsg>(10);

    Ok(Self {
      config: module_config,
      // module,
      // linker,
      engine,
      instance_pre,
      host_channel: Arc::new(Mutex::new((send_2, recv_1))),
      module_channel: Arc::new(Mutex::new((send_1, recv_2))),
    })
  }

  async fn cb_call_service(
    mut caller: Caller<'_, ServiceStore>,
    name_ptr: i32,
    data_ptr: i32,
  ) -> Result<i32> {
    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
      return Err(Error::msg("Memory export not defined"));
    };
    let name_buf = ServiceStore::read_from_memory(&caller.as_context(), memory, name_ptr)?;
    let name = String::from_utf8(name_buf)?;

    let data = ServiceStore::read_from_memory(&caller.as_context(), memory, data_ptr)?;

    let response_data: Result<Vec<u8>> = {
      let mut channel = caller.as_context().data().channel.lock().await;
      channel.0.send(SendMsg::Data { name, data }).await?;
      // Wait for response
      let msg = channel.1.recv().await.ok_or(Error::msg("No data recv"))?;
      Ok(msg.data)
    };

    let resp_ptr =
      ServiceStore::write_to_memory(&mut caller.as_context_mut(), memory, response_data?)?;

    Ok(resp_ptr)
  }

  fn cb_send_response(mut caller: Caller<'_, ServiceStore>, ptr: i32) -> Result<()> {
    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
      return Err(Error::msg("Memory export not defined"));
    };
    let mut buffer = ServiceStore::read_from_memory(&caller.as_context(), memory, ptr)?;
    caller.data_mut().response_data.append(&mut buffer);
    Ok(())
  }

  fn cb_set_response_metadata(
    mut caller: Caller<'_, ServiceStore>,
    key_ptr: i32,
    value_ptr: i32,
  ) -> Result<()> {
    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
      return Err(Error::msg("Memory export not defined"));
    };

    let key_buf = ServiceStore::read_from_memory(&caller.as_context(), memory, key_ptr)?;
    let key = String::from_utf8(key_buf)?;

    let value_buf = ServiceStore::read_from_memory(&caller.as_context(), memory, value_ptr)?;
    let value = String::from_utf8(value_buf)?;

    caller.data_mut().response_metadata.insert(key, value);

    Ok(())
  }

  fn cb_get_metadata(mut caller: Caller<'_, ServiceStore>, ptr: i32) -> Result<i32> {
    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
      return Err(Error::msg("Memory export not defined"));
    };

    let key_buf = ServiceStore::read_from_memory(&caller.as_context(), memory, ptr)?;
    let key = String::from_utf8(key_buf)?;
    let metadata = &caller.data().metadata;

    let value: Vec<u8> = metadata
      .get(&key)
      .map(|s| s.to_owned())
      .unwrap_or_default()
      .into();

    let value_ptr = ServiceStore::write_to_memory(&mut caller.as_context_mut(), memory, value)?;

    Ok(value_ptr)
  }

  pub async fn instantiate(&self, request_handler_name: &str) -> Result<ServiceInstance> {
    ServiceInstance::new(self, request_handler_name).await
  }
}

pub struct ServiceInstance {
  instance: Instance,
  request_handler_name: String,
  store: Store<ServiceStore>,
}

impl ServiceInstance {
  pub async fn new(service: &ServiceModule, request_handler_name: &str) -> Result<Self> {
    let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();
    let mut store = Store::new(
      &service.engine,
      ServiceStore {
        wasi_ctx,
        metadata: HashMap::new(),
        response_metadata: HashMap::new(),
        response_data: Vec::new(),
        pointer_offset: 1,
        channel: service.module_channel.clone(),
      },
    );

    Memory::new(&mut store, MemoryType::new(1, None))?;

    let instance = service.instance_pre.instantiate_async(&mut store).await?;

    // Grow memory size
    let memory = instance
      .get_memory(&mut store, "memory")
      .ok_or(Error::msg("No memory of anything"))?;
    memory.grow_async(&mut store, 1000).await?;
    dbg!(memory.size(&mut store));

    if instance.get_export(&mut store, "_start").is_some() {
      let initialize = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
      initialize.call_async(&mut store, ()).await?;
    }

    Ok(ServiceInstance {
      request_handler_name: request_handler_name.to_string(),
      instance,
      store,
    })
  }

  pub async fn invoke(&mut self, data: Vec<u8>) -> Result<()> {
    let ptr = self.write_to_memory(data)?;
    let on_request = self
      .instance
      .get_typed_func::<i32, ()>(&mut self.store, &self.request_handler_name)?;

    on_request.call_async(&mut self.store, ptr).await?;

    let channel = Arc::clone(&self.store.data().channel);
    tokio::spawn(async move {
      // TODO: An extra delay just-in-case. Remove later
      tokio::time::sleep(std::time::Duration::from_millis(100)).await;
      let ch = channel.lock().await;
      ch.0.send(SendMsg::End).await.expect("unable to end task");
    });

    Ok(())
  }

  pub fn write_to_memory(&mut self, data: Vec<u8>) -> Result<i32> {
    let memory = self
      .instance
      .get_memory(&mut self.store, "memory")
      .ok_or(Error::msg("Memory export not defined"))?;

    ServiceStore::write_to_memory(&mut self.store.as_context_mut(), memory, data)
  }

  pub fn update_metadata(&mut self, map: HashMap<String, String>) {
    let data = self.store.data_mut();
    data.metadata.extend(map)
  }

  pub fn get_response_metadata(&self) -> &HashMap<String, String> {
    &self.store.data().response_metadata
  }

  pub fn get_response_data(&self) -> &Vec<u8> {
    &self.store.data().response_data
  }
}
