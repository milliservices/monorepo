use std::sync::Arc;

use milliservices_core::node;
use milliservices_core::service::ModuleConfig;

#[tokio::test]
async fn simple_node_single_module_1() {
  let node = node::Node::new_ref();

  let cfg = ModuleConfig {
    name: "test-module".to_string(),
    path: "../../test-fixtures/module-1/lib.wasm".to_string(),
    symbol: "simple_calculations".to_string(),
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
async fn simple_node_single_module_2() {
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
