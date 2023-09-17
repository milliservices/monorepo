#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

use anyhow::{Error, Result};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use milliservices_core::node;
use milliservices_core::service;

#[tokio::main]
async fn main() -> Result<()> {
  let node_ref = node::Node::new_ref();

  let module_configs = vec![
    service::ModuleConfig {
      name: "/rust".to_string(),
      path: "./target/wasm32-wasi/debug/example_rust_wasm.wasm".to_string(),
      symbol: "on_request".to_string(),
      ..Default::default()
    },
    service::ModuleConfig {
      name: "/ass".to_string(),
      path: "./examples/assemblyscript/build/debug.wasm".to_string(),
      symbol: "on_request".to_string(),
      ..Default::default()
    },
    service::ModuleConfig {
      name: "/rust-final".to_string(),
      path: "./target/wasm32-wasi/debug/example_rust_wasm.wasm".to_string(),
      symbol: "final_call".to_string(),
      ..Default::default()
    },
    service::ModuleConfig {
      name: "/haskell".to_string(),
      path: "./examples/haskell/build/lib.wasm".to_string(),
      symbol: "onRequest".to_string(),
      ..Default::default()
    },
  ];

  for cfg in module_configs {
    node_ref.lock().await.load_module(cfg).await?;
  }

  let debug_start_time = std::time::Instant::now();
  run_instance_test(node_ref, "/rust".to_string()).await?;
  dbg!(debug_start_time.elapsed());

  Ok(())
}

async fn run_instance_test(node_ref: Arc<Mutex<node::Node>>, name: String) -> Result<()> {
  let mut instance = node::spawn_instance(Arc::clone(&node_ref), name)
    .await?
    .ok_or(Error::msg("foobar"))?;

  instance.update_metadata(HashMap::from([
    ("@method".to_string(), "POST".to_string()),
    ("@path".to_string(), "/some/path".to_string()),
    ("authentication".to_string(), "some auth key".to_string()),
  ]));
  instance.invoke("Request data incoming".into()).await?;

  dbg!(instance.get_response_metadata());
  let response = String::from_utf8(instance.get_response_data().to_owned());
  let _ = dbg!(response);

  Ok(())
}
