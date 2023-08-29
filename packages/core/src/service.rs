use anyhow::Result;
use std::collections::HashMap;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

use crate::service_store::ServiceStore;

pub struct ServiceModule {
  // module: Module,
  engine: Engine,
  // linker: Linker<ServiceStore>,
  instance_pre: InstancePre<ServiceStore>,
}

impl ServiceModule {
  pub async fn new(path: &str) -> Result<Self> {
    let mut config = Config::default();
    config.async_support(true);
    let engine = Engine::new(&config)?;
    let module = Module::from_file(&engine, path)?;

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

    let instance_pre = linker.instantiate_pre(&module)?;

    Ok(Self {
      // module,
      engine,
      // linker,
      instance_pre,
    })
  }

  fn cb_send_response(mut caller: Caller<'_, ServiceStore>, ptr: i32) -> Result<()> {
    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
      panic!("Memory export not defined")
    };
    let buffer = ServiceStore::read_from_memory(&caller.as_context(), memory, ptr)?;
    let _ = dbg!(String::from_utf8(buffer));
    Ok(())
  }

  fn cb_set_response_metadata(
    mut caller: Caller<'_, ServiceStore>,
    key_ptr: i32,
    value_ptr: i32,
  ) -> Result<()> {
    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
      panic!("Memory export not defined")
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
      panic!("Memory export not defined")
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

  pub async fn instantiate(self) -> Result<ServiceInstance> {
    let inst = ServiceInstance::new(self).await?;
    Ok(inst)
  }
}

pub struct ServiceInstance {
  instance: Instance,
  store: Store<ServiceStore>,
}

impl ServiceInstance {
  pub async fn new(service: ServiceModule) -> Result<Self> {
    let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();
    let mut store = Store::new(
      &service.engine,
      ServiceStore {
        wasi_ctx,
        metadata: HashMap::new(),
        response_metadata: HashMap::new(),
        pointer_offset: 1,
      },
    );

    Memory::new(&mut store, MemoryType::new(1, None))?;

    let instance = service.instance_pre.instantiate_async(&mut store).await?;

    Ok(ServiceInstance { instance, store })
  }

  pub async fn invoke(&mut self, ptr: i32) -> Result<()> {
    let on_request = self
      .instance
      .get_typed_func::<i32, ()>(&mut self.store, "on_request")?;

    on_request.call_async(&mut self.store, ptr).await?;

    Ok(())
  }

  pub fn write_to_memory(&mut self, data: Vec<u8>) -> Result<i32> {
    let memory = self
      .instance
      .get_memory(&mut self.store, "memory")
      .expect("No memory of anything");

    ServiceStore::write_to_memory(&mut self.store.as_context_mut(), memory, data)
  }

  pub fn update_metadata(&mut self, map: HashMap<String, String>) {
    let data = self.store.data_mut();
    data.metadata.extend(map)
  }

  pub fn get_response_metadata(&self) -> &HashMap<String, String> {
    &self.store.data().response_metadata
  }
}
