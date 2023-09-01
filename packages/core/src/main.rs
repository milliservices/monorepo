#![feature(type_alias_impl_trait)]

use anyhow::Result;
use node::TaskHandler;
use std::collections::HashMap;

pub mod node;
pub mod service;
pub mod store;

use service::ModuleConfig;

#[tokio::main]
async fn main() -> Result<()> {
  let mut node = node::Node::new();
  let module_configs = vec![
    ModuleConfig {
      path: "./target/wasm32-wasi/debug/example_rust_wasm.wasm".to_string(),
      name: "rust".to_string(),
      symbol: "on_request".to_string(),
    },
    ModuleConfig {
      path: "./packages/example-assemblyscript/build/debug.wasm".to_string(),
      name: "ass".to_string(),
      symbol: "on_request".to_string(),
    },
  ];

  for cfg in module_configs {
    node.load_module(cfg).await?;
  }

  let task_handler = node.launch_handler();

  let debug_start_time = std::time::Instant::now();
  run_instance(&mut node, "rust").await?;
  // run_instance(&mut node, "ass").await?;
  dbg!(debug_start_time.elapsed());

  task_handler.await;

  Ok(())
}

async fn run_instance(node: &mut node::Node, name: &str) -> Result<()> {
  let mut instance = node.create_instance(name.to_string()).await?;

  instance.update_metadata(HashMap::from([
    ("@method".to_string(), "POST".to_string()),
    ("@path".to_string(), "/some/path".to_string()),
    ("X-Authentication".to_string(), "some auth key".to_string()),
  ]));
  instance.invoke("Request data incoming".into()).await?;

  dbg!(instance.get_response_metadata());
  let _ = dbg!(String::from_utf8(instance.get_response_data().to_owned()));

  Ok(())
}

async fn example_assemblyscript(node: &mut node::Node) -> Result<TaskHandler> {
  let cfg = ModuleConfig {
    path: "./packages/example-assemblyscript/build/debug.wasm".to_string(),
    name: "foobar".to_string(),
    symbol: "on_request".to_string(),
  };
  let name = cfg.name.to_string();
  node.load_module(cfg).await?;

  let mut instance = node.create_instance(name).await?;
  let task_handler = node.launch_handler();

  instance.invoke("Request data incoming".into()).await?;

  dbg!(instance.get_response_metadata());
  let _ = dbg!(String::from_utf8(instance.get_response_data().to_owned()));

  Ok(task_handler)
}
