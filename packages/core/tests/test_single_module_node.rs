use milliservices_core::service::ModuleConfig;

#[path = "./utils.rs"]
mod utils;

#[tokio::test]
async fn module_1_simple_io() {
  let (mut instance, _) = utils::get_module_instance(ModuleConfig {
    path: utils::get_wasm_path("module-1"),
    symbol: "simple_io".to_string(),
    ..Default::default()
  })
  .await;

  instance.initialize().await.expect("init failed");

  instance
    .invoke("Giving input".into())
    .await
    .expect("invoke");

  let response = String::from_utf8(instance.get_response_data().to_owned());
  assert_eq!(response, Ok("Getting output after Giving input".into()));
}

#[tokio::test]
async fn module_2_simple_io() {
  let (mut instance, _) = utils::get_module_instance(ModuleConfig {
    path: utils::get_wasm_path("module-2"),
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
  let (mut instance, _) = utils::get_module_instance(ModuleConfig {
    path: utils::get_wasm_path("module-2"),
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
