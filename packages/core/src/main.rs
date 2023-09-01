use anyhow::Result;
use std::collections::HashMap;
use tokio::task::JoinHandle;

pub mod node;
pub mod service;
pub mod store;

use service::ModuleConfig;

#[tokio::main]
async fn main() -> Result<()> {
  let mut node = node::Node::new();

  let debug_start_time = std::time::Instant::now();

  // let task_handler = example_rust(&mut node).await?;
  let task_handler = example_rust(&mut node).await?;

  dbg!(debug_start_time.elapsed());

  task_handler.await??;
  // let re = tokio::join!(task_handler_1, task_handler_2);
  // println!("{re:?}");

  Ok(())
}

async fn example_rust(node: &mut node::Node) -> Result<JoinHandle<Result<()>>> {
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

  Ok(task_handler)
}

async fn example_assemblyscript(node: &mut node::Node) -> Result<JoinHandle<Result<()>>> {
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
