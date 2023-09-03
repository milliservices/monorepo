use std::sync::Arc;

use milliservices_core::node;
use milliservices_core::service::ModuleConfig;

#[tokio::test]
async fn module_1_simple_io() {
  let node = node::Node::new_ref();

  let cfg = ModuleConfig {
    name: "test-module".to_string(),
    path: "../../test-fixtures/module-1/lib.wasm".to_string(),
    symbol: "simple_io".to_string(),
    ..Default::default()
  };

  node.lock().await.load_module(cfg).await.expect("module");

  let mut instance = node::spawn_instance(Arc::clone(&node), "test-module".to_string())
    .await
    .expect("instance");

  instance.invoke(vec![6, 9, 4, 2, 0]).await.expect("invoke");

  let response = instance.get_response_data().to_owned();

  assert_eq!(response, vec![] as Vec<u8>);
}

#[tokio::test]
async fn module_2_simple_io() {
  let node = node::Node::new_ref();

  let cfg = ModuleConfig {
    name: "test-module".to_string(),
    path: "../../test-fixtures/module-2/lib.wasm".to_string(),
    symbol: "simple_io".to_string(),
    ..Default::default()
  };

  node.lock().await.load_module(cfg).await.expect("module");

  let mut instance = node::spawn_instance(Arc::clone(&node), "test-module".to_string())
    .await
    .expect("instance");

  instance.invoke("input data".into()).await.expect("invoke");

  let response_buf = instance.get_response_data().to_owned();
  let response = String::from_utf8(response_buf);

  assert_eq!(response, Ok("input data. adds output data".to_string()));
}

#[tokio::test]
async fn module_2_simple_calculations() {
  let node = node::Node::new_ref();

  let cfg = ModuleConfig {
    name: "test-module".to_string(),
    path: "../../test-fixtures/module-2/lib.wasm".to_string(),
    symbol: "simple_calculations".to_string(),
    ..Default::default()
  };

  node.lock().await.load_module(cfg).await.expect("module");

  let mut instance = node::spawn_instance(Arc::clone(&node), "test-module".to_string())
    .await
    .expect("instance");

  let mut a = (2900i32).to_le_bytes().to_vec();
  let mut b = (1300i32).to_le_bytes().to_vec();
  a.append(&mut b);
  instance.invoke(a).await.expect("invoke");

  let mut buf = [0u8; 4];
  let response_buf = instance.get_response_data().to_owned();
  buf.copy_from_slice(response_buf.as_slice());
  let response = i32::from_le_bytes(buf);

  assert_eq!(response, 4200);
}
