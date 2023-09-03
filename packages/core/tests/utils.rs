use milliservices_core::node::{self, NodeRef};
use milliservices_core::service::{ModuleConfig, ServiceInstance};
use std::sync::Arc;

pub async fn get_module_instance(cfg: ModuleConfig) -> (ServiceInstance, NodeRef) {
  let node = node::Node::new_ref();

  let cfg = ModuleConfig {
    name: "test-module".to_string(),
    ..cfg
  };

  node.lock().await.load_module(cfg).await.expect("module");

  let instance = node::spawn_instance(Arc::clone(&node), "test-module".to_string())
    .await
    .expect("instance");

  (instance, node)
}

pub fn get_wasm_path(s: &str) -> String {
  format!("../../test-fixtures/{s}/lib.wasm")
}
