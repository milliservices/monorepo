extern "C" {
  fn send_response(input_ptr: i32, input_size: u32);
}

// fn get_metadata_ptr(key_ptr: i32, key_size: u32) -> i32;
// fn get_metadata_size(key_ptr: i32, key_size: u32) -> u32;
//
// i32->u32
// fn get_metadata(key: String) -> String;

fn get_mem_representation(ptr: i32) -> (i32, usize) {
  let mut buf = [0u8; 4];
  let data_ptr_buf = unsafe { std::slice::from_raw_parts(ptr as *mut u8, 4) };
  buf.copy_from_slice(data_ptr_buf);
  let data_ptr = i32::from_be_bytes(buf);

  let data_len_buf = unsafe { std::slice::from_raw_parts((ptr + 4) as *mut u8, 4) };
  buf.copy_from_slice(data_len_buf);
  let data_len = u32::from_be_bytes(buf);

  (data_ptr, data_len as usize)
}

fn get_string_from_ptr(in_ptr: i32) -> String {
  let (ptr, len) = get_mem_representation(in_ptr);
  let data_str = unsafe { String::from_raw_parts(ptr as *mut u8, len, len) };
  let str = data_str.clone();
  std::mem::forget(data_str);

  str
}

#[no_mangle]
extern "C" fn on_request(input_ptr: i32) {
  let str = get_string_from_ptr(input_ptr);
  dbg!(str);

  unsafe {
    let some_buf: Vec<u8> = "FROM WASM".into();
    let ptr = some_buf.as_ptr();
    send_response(ptr as i32, some_buf.len() as u32);
  }
}
