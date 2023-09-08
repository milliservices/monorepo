use milliservices_core::{node, service};
use milliservices_http_layer::HttpLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
  ];

  let node = node::Node::new_ref();

  for cfg in module_configs {
    node.lock().await.load_module(cfg).await?;
  }

  let server = HttpLayer::new(node);

  server.init(([127, 0, 0, 1], 3000).into()).await?;

  Ok(())
}
