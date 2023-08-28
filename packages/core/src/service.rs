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

    linker.func_wrap(
      "env",
      "send_response",
      |mut caller: Caller<'_, ServiceStore>, ptr: i32| -> Result<()> {
        if let Some(Extern::Memory(memory)) = caller.get_export("memory") {
          let buffer = ServiceStore::read_from_memory(&caller.as_context(), memory, ptr)?;
          let _ = dbg!(String::from_utf8(buffer));
        }
        Ok(())
      },
    )?;

    let instance_pre = linker.instantiate_pre(&module)?;

    Ok(Self {
      // module,
      engine,
      // linker,
      instance_pre,
    })
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

  pub fn encode_ptr(&mut self, data: Vec<u8>) -> Result<i32> {
    let memory = self
      .instance
      .get_memory(&mut self.store, "memory")
      .expect("No memory of anything");

    let mut ctx = self.store.as_context_mut();
    ServiceStore::write_to_memory(&mut ctx, memory, data)
  }
}
