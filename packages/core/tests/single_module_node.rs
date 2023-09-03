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

#[tokio::test]
async fn module_1_simple_io() {
  let (mut instance, _) = get_module_instance(ModuleConfig {
    path: "../../test-fixtures/module-1/lib.wasm".to_string(),
    symbol: "simple_io".to_string(),
    ..Default::default()
  })
  .await;

  instance.invoke(vec![6, 9, 4, 2, 0]).await.expect("invoke");

  let response = instance.get_response_data().to_owned();
  assert_eq!(response, vec![] as Vec<u8>);
}

#[tokio::test]
async fn module_2_simple_io() {
  let (mut instance, _) = get_module_instance(ModuleConfig {
    path: "../../test-fixtures/module-2/lib.wasm".to_string(),
    symbol: "simple_io".to_string(),
    ..Default::default()
  })
  .await;

  instance.invoke("input data".into()).await.expect("invoke");

  let response_buf = instance.get_response_data().to_owned();
  let response = String::from_utf8(response_buf);
  assert_eq!(response, Ok("input data. adds output data".to_string()));
}

#[tokio::test]
async fn module_2_simple_calculations() {
  let (mut instance, _) = get_module_instance(ModuleConfig {
    path: "../../test-fixtures/module-2/lib.wasm".to_string(),
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
