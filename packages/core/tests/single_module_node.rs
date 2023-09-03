use milliservices_core::node::{self, NodeRef};
use milliservices_core::service::{ModuleConfig, ServiceInstance};
use std::sync::Arc;

async fn get_module_instance(cfg: ModuleConfig) -> (ServiceInstance, NodeRef) {
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

fn get_wasm_path(s: &str) -> String { format!("../../test-fixtures/{s}/lib.wasm") }

#[tokio::test]
async fn module_1_simple_io() {
  let (mut instance, _) = get_module_instance(ModuleConfig {
    path: get_wasm_path("module-1"),
    symbol: "simple_io".to_string(),
    ..Default::default()
  })
  .await;

  instance.invoke("Giving input".into()).await.expect("invoke");

  let response = String::from_utf8(instance.get_response_data().to_owned());
  assert_eq!(response, Ok("Getting output after Giving input".into()));
}

#[tokio::test]
async fn module_2_simple_io() {
  let (mut instance, _) = get_module_instance(ModuleConfig {
    path: get_wasm_path("module-2"),
    symbol: "simple_io".to_string(),
    ..Default::default()
  })
  .await;

  instance.invoke("input data".into()).await.expect("invoke");

  let response = String::from_utf8(instance.get_response_data().to_owned());
  assert_eq!(response, Ok("input data. adds output data".to_string()));
}

#[tokio::test]
async fn module_2_simple_calculations() {
  let (mut instance, _) = get_module_instance(ModuleConfig {
    path: get_wasm_path("module-2"),
    symbol: "simple_calculations".to_string(),
    ..Default::default()
  })
  .await;

  let mut a = (2900_i32).to_le_bytes().to_vec();
  let mut b = (1300_i32).to_le_bytes().to_vec();
  a.append(&mut b);
  instance.invoke(a).await.expect("invoke");

  let mut buf = [0u8; 4];
  buf.copy_from_slice(instance.get_response_data().to_owned().as_slice());
  let response = i32::from_le_bytes(buf);
  assert_eq!(response, 4200_i32);
}
