use std::sync::Arc;

use milliservices_core::node;
use milliservices_core::service::ModuleConfig;

#[tokio::test]
async fn single_module_node() {
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
