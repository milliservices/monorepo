use milliservices_core::node;
use milliservices_core::service::ModuleConfig;

#[tokio::test]
async fn single_module_node() {
  let mut node = node::Node::new();
  let cfg = ModuleConfig {
    name: "test-module".to_string(),
    path: "../../test-fixtures/module-1/lib.wasm".to_string(),
    symbol: "simple_calculations".to_string(),
  };

  node.load_module(cfg).await.expect("module");

  let mut instance = node
    .create_instance("test-module".to_string())
    .await
    .expect("instance");

  instance.invoke(vec![6, 9, 4, 2, 0]).await.expect("invoke");

  let response = instance.get_response_data().to_owned();

  assert_eq!(response, vec![] as Vec<u8>);
}
