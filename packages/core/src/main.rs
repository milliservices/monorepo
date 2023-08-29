use std::collections::HashMap;

use anyhow::Result;

pub mod service;
pub mod service_store;

use service::ServiceModule;

#[tokio::main]
async fn main() -> Result<()> {
  let service_module =
    ServiceModule::new("./target/wasm32-wasi/debug/example_rust_wasm.wasm").await?;

  let mut instance = service_module.instantiate("on_request").await?;

  let pointer = instance.write_to_memory("Hello world".into())?;
  instance.update_metadata(HashMap::from([
    ("@method".to_string(), "POST".to_string()),
    ("@path".to_string(), "/some/path".to_string()),
    ("X-Authentication".to_string(), "some auth key".to_string()),
  ]));
  instance.invoke(pointer).await?;

  let response_metadata = instance.get_response_metadata();
  dbg!(response_metadata);

  Ok(())
}
