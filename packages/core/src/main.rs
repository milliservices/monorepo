use anyhow::Result;
use std::collections::HashMap;

pub mod service;
pub mod service_store;

use service::{ModuleConfig, Node};

#[tokio::main]
async fn main() -> Result<()> {
  let mut node = Node::new();

  let cfg = ModuleConfig {
    path: "./target/wasm32-wasi/debug/example_rust_wasm.wasm".to_string(),
    name: "foobar".to_string(),
    symbol: "on_request".to_string(),
  };
  let name = cfg.name.to_string();
  node.load_module(cfg).await?;

  let mut instance = node.create_instance(name).await?;

  let task_handler = node.launch_handler();

  instance.update_metadata(HashMap::from([
    ("@method".to_string(), "POST".to_string()),
    ("@path".to_string(), "/some/path".to_string()),
    ("X-Authentication".to_string(), "some auth key".to_string()),
  ]));
  instance.invoke("Request data incoming".into()).await?;

  dbg!(instance.get_response_metadata());
  let _ = dbg!(String::from_utf8(instance.get_response_data().to_owned()));

  task_handler.await??;

  Ok(())
}
