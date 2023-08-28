use anyhow::Result;
use std::collections::HashMap;
use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

struct ServiceStore {
  pub wasi_ctx: WasiCtx,
  pub metadata: HashMap<String, String>,
}

struct ServiceModule {
  // module: Module,
  engine: Engine,
  // linker: Linker<ServiceStore>,
  instance_pre: InstancePre<ServiceStore>,
}

impl ServiceModule {
  async fn new(path: &str) -> Result<Self> {
    let mut config = Config::default();
    config.async_support(true);
    let engine = Engine::new(&config)?;
    let module = Module::from_file(&engine, path)?;

    let mut linker: Linker<ServiceStore> = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |store: &mut ServiceStore| &mut store.wasi_ctx)?;

    linker.func_wrap(
      "env",
      "send_response",
      |mut caller: Caller<'_, ServiceStore>, ptr: i32, size: u32| -> Result<()> {
        if let Some(Extern::Memory(memory)) = caller.get_export("memory") {
          let mut buffer = vec![0u8; size as usize];
          memory.read(&mut caller, ptr as usize, &mut buffer)?;
          println!("Got from WebAssembly {:?}", String::from_utf8(buffer));
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

  async fn instance(self) -> Result<ServiceInstance> {
    let inst = ServiceInstance::new(self).await?;
    Ok(inst)
  }
}

pub struct ServiceInstance {
  instance: Instance,
  store: Store<ServiceStore>,
  current_pointer_cursor: u32,
}

impl ServiceInstance {
  async fn new(service: ServiceModule) -> Result<Self> {
    let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();
    let mut store = Store::new(
      &service.engine,
      ServiceStore {
        wasi_ctx,
        metadata: HashMap::new(),
      },
    );

    Memory::new(&mut store, MemoryType::new(1, None))?;

    let instance = service.instance_pre.instantiate_async(&mut store).await?;

    Ok(ServiceInstance {
      instance,
      store,
      current_pointer_cursor: 1,
    })
  }

  async fn invoke(&mut self, ptr: i32) -> Result<()> {
    let on_request = self
      .instance
      .get_typed_func::<i32, ()>(&mut self.store, "on_request")?;

    on_request.call_async(&mut self.store, ptr).await?;

    Ok(())
  }

  fn write_to_memory(&mut self, data: Vec<u8>) -> Result<u32> {
    let memory = self
      .instance
      .get_memory(&mut self.store, "memory")
      .expect("Nothing");

    let data_ptr = self.current_pointer_cursor;
    memory.write(&mut self.store, data_ptr as usize, data.as_slice())?;

    // TODO: Cyclic incremenet, maybe?
    self.current_pointer_cursor += data.len() as u32;

    Ok(data_ptr)
  }

  fn encode_ptr(&mut self, data: Vec<u8>) -> Result<i32> {
    let data_len = data.len();

    // Write data to memory
    let data_ptr = self.write_to_memory(data)?;

    let mut ptr_ptr_buf: Vec<u8> = data_ptr.to_be_bytes().into();
    let mut ptr_len_buf: Vec<u8> = (data_len as u32).to_be_bytes().into();
    ptr_ptr_buf.append(&mut ptr_len_buf);

    let ptr = self.write_to_memory(ptr_ptr_buf)?;

    Ok(ptr as i32)
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let service_module =
    ServiceModule::new("./target/wasm32-wasi/debug/example_rust_wasm.wasm").await?;

  let mut instance = service_module.instance().await?;

  let pointer = instance.encode_ptr("Hello world".into())?;
  instance.invoke(pointer).await?;

  Ok(())
}
