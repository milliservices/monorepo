use anyhow::Result;
use std::collections::HashMap;

use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

use crate::imports;
use crate::store::{HandleCallService, ServiceStore};

#[derive(Debug, Clone)]
pub struct ModuleSystem {
  pub memory_pages: u32,
}

impl Default for ModuleSystem {
  fn default() -> Self {
    Self { memory_pages: 100 }
  }
}

#[derive(Debug, Clone)]
pub struct ModuleConfig {
  pub path: String,
  pub symbol: String,
  pub name: String,
  pub system: ModuleSystem,
}

impl Default for ModuleConfig {
  fn default() -> Self {
    Self {
      path: "".to_string(),
      symbol: "on_request".to_string(),
      name: "".to_string(), // TODO: auto generate uuid?
      system: ModuleSystem::default(),
    }
  }
}

pub struct ServiceModule {
  pub config: ModuleConfig,
  engine: Engine,
  instance_pre: InstancePre<ServiceStore>,
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

    let host_module_name = "milliservices_v1";

    linker.func_wrap(host_module_name, "send_response", imports::send_response)?;
    linker.func_wrap(host_module_name, "get_metadata", imports::get_metadata)?;
    linker.func_wrap(
      host_module_name,
      "set_response_metadata",
      imports::set_response_metadata,
    )?;
    linker.func_wrap2_async(
      host_module_name,
      "call_service",
      |caller, name_ptr: i32, data_ptr: i32| {
        Box::new(imports::call_service(caller, name_ptr, data_ptr))
      },
    )?;

    linker.func_wrap(
      host_module_name,
      "service_new_request",
      imports::service_new_request,
    )?;
    linker.func_wrap(
      host_module_name,
      "service_write_data",
      imports::service_write_data,
    )?;
    linker.func_wrap(
      host_module_name,
      "service_get_response",
      imports::service_get_response,
    )?;
    linker.func_wrap(
      host_module_name,
      "service_set_metadata",
      imports::service_set_metadata,
    )?;
    linker.func_wrap(
      host_module_name,
      "service_get_response_metadata",
      imports::service_get_response_metadata,
    )?;
    linker.func_wrap1_async(
      host_module_name,
      "service_execute",
      |caller, req_id: u32| Box::new(imports::service_execute(caller, req_id)),
    )?;

    let instance_pre = linker.instantiate_pre(&module)?;

    Ok(Self {
      config: module_config,
      engine,
      instance_pre,
      // module,
      // linker,
    })
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
        handle_call_service: None,
        requests: HashMap::new(),
        request_count: 0,
      },
    );

    Memory::new(&mut store, MemoryType::new(1, None))?;

    let instance = service.instance_pre.instantiate_async(&mut store).await?;

    // Grow memory size
    let memory = instance
      .get_memory(&mut store, "memory")
      .ok_or(Error::msg("No memory of anything"))?;
    memory
      .grow_async(&mut store, service.config.system.memory_pages as u64)
      .await?;
    dbg!(memory.size(&mut store));

    Ok(ServiceInstance {
      request_handler_name: request_handler_name.to_string(),
      instance,
      store,
    })
  }

  pub async fn set_call_service_handler(&mut self, cb: HandleCallService) {
    let data = self.store.data_mut();
    data.handle_call_service = Some(cb);
  }

  pub async fn initialize(&mut self) -> Result<()> {
    let instance = &self.instance;
    let mut store = &mut self.store;

    // WASI module instantiation
    if let Ok(init_fn) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
      init_fn.call_async(&mut store, ()).await?;
    } else if let Ok(init_fn) = instance.get_typed_func::<(), ()>(&mut store, "_initialize") {
      init_fn.call_async(&mut store, ()).await?;
    }

    // NOTE: Support for haskell's hs_init call
    if let Ok(init_fn) = instance.get_typed_func::<(i32, i32), ()>(&mut store, "hs_init") {
      init_fn.call_async(&mut store, (0, 0)).await?;
    }

    Ok(())
  }

  pub async fn invoke(&mut self, data: Vec<u8>) -> Result<()> {
    let memory = self
      .instance
      .get_memory(&mut self.store, "memory")
      .ok_or(Error::msg("Memory export not defined"))?;

    let ptr = ServiceStore::write_to_memory(&mut self.store.as_context_mut(), memory, data)?;

    let on_request = self
      .instance
      .get_typed_func::<i32, ()>(&mut self.store, &self.request_handler_name)?;

    on_request.call_async(&mut self.store, ptr).await?;

    Ok(())
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
