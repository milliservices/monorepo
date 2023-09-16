#[link(wasm_import_module = "milliservices_v1")]
extern "C" {
  pub fn send_response(data_ptr: i32);
  pub fn get_metadata(key_ptr: i32) -> i32;
  pub fn set_response_metadata(key_ptr: i32, value_ptr: i32);

  // Service
  pub fn service_new_request(ptr: i32) -> u32;
  pub fn service_execute(id: u32);
  pub fn service_write_data(id: u32, ptr: i32);
  pub fn service_get_response(id: u32) -> i32;
  pub fn service_set_metadata(id: u32, key_ptr: i32, value_ptr: i32);
  pub fn service_get_response_metadata(id: u32, key_ptr: i32) -> i32;
}
