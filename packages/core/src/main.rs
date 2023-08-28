use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

fn main() -> wasmtime::Result<()> {
  let engine = Engine::default();
  let module = Module::from_file(&engine, "./target/wasm32-wasi/debug/example_rust_wasm.wasm")?;

  let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();
  let mut store = Store::new(&engine, wasi_ctx);

  let memory_ty = MemoryType::new(1, None);
  Memory::new(&mut store, memory_ty)?;

  let mut linker = Linker::new(&engine);
  wasmtime_wasi::add_to_linker(&mut linker, |cx| cx)?;

  linker.func_wrap(
    "env",
    "send_response",
    |_caller: Caller<'_, WasiCtx>, ptr: i32, size: u32| {
      println!("Got from WebAssembly {ptr} {size}");
    },
  )?;

  let instance = linker.instantiate(&mut store, &module)?;
  let memory = instance.get_memory(&mut store, "memory").expect("Nothing");

  let pointer: usize = 1;
  let buffer = &[1, 3, 3, 7];
  memory.write(&mut store, pointer, buffer)?;

  let foobar = instance.get_typed_func::<(i32, u32), ()>(&mut store, "on_request")?;
  foobar.call(&mut store, (pointer as i32, buffer.len() as u32))?;

  Ok(())
}
