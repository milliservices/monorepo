use anyhow::Result;

pub mod service;
pub mod service_store;

use service::ServiceModule;

#[tokio::main]
async fn main() -> Result<()> {
  let service_module =
    ServiceModule::new("./target/wasm32-wasi/debug/example_rust_wasm.wasm").await?;

  let mut instance = service_module.instantiate().await?;

  let pointer = instance.encode_ptr("Hello world".into())?;
  instance.invoke(pointer).await?;

  Ok(())
}
