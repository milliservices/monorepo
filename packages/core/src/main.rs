#![feature(type_alias_impl_trait)]

use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub mod node;
pub mod service;
pub mod store;

use service::ModuleConfig;

#[tokio::main]
async fn main() -> Result<()> {
  let node_ref = node::Node::new_ref();

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
    ModuleConfig {
      path: "./target/wasm32-wasi/debug/example_rust_wasm.wasm".to_string(),
      name: "rust".to_string(),
      symbol: "final_call".to_string(),
    },
  ];

  for cfg in module_configs {
    node_ref.lock().await.load_module(cfg).await?;
  }

  let task_handler = node::launch_node_msg_handler(Arc::clone(&node_ref)).await;

  let debug_start_time = std::time::Instant::now();
  run_instance_test(node_ref, "rust".to_string()).await?;
  dbg!(debug_start_time.elapsed());

  for res in task_handler.await {
    res??;
  }

  Ok(())
}

async fn run_instance_test(node_ref: Arc<Mutex<node::Node>>, name: String) -> Result<()> {
  let mut instance = node::create_instance(Arc::clone(&node_ref), name).await?;

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
