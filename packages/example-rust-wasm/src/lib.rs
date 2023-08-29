use std::mem::size_of;

extern "C" {
  fn send_response(data_ptr: i32);
  fn get_metadata(key_ptr: i32) -> i32;
  // fn set_response_metadata(key_ptr: i32, value_ptr: i32);
  // fn call_service(name_ptr: i32, data_ptr: i32, metadata??) -> i32;
}

fn get_mem_representation(ptr: i32) -> (i32, usize) {
  let mut buf = [0u8; size_of::<u32>()];

  let data_ptr_buf = unsafe { std::slice::from_raw_parts(ptr as *mut u8, size_of::<i32>()) };
  buf.copy_from_slice(data_ptr_buf);
  let data_ptr = i32::from_be_bytes(buf);

  let data_len_buf = unsafe { std::slice::from_raw_parts((ptr + 4) as *mut u8, size_of::<u32>()) };
  buf.copy_from_slice(data_len_buf);
  let data_len = u32::from_be_bytes(buf);

  (data_ptr, data_len as usize)
}

fn read_from_memory(ptr: i32) -> Vec<u8> {
  let (data_ptr, data_len) = get_mem_representation(ptr);
  let data_str = unsafe { Vec::from_raw_parts(data_ptr as *mut u8, data_len, data_len) };
  let str = data_str.clone();
  std::mem::forget(data_str);

  str
}

fn write_to_memory(data: Vec<u8>) -> i32 {
  let data_len = data.len();
  let data_ptr = data.as_ptr() as i32;

  let mut ptr_ptr_buf: Vec<u8> = data_ptr.to_be_bytes().into();
  let mut ptr_len_buf: Vec<u8> = (data_len as u32).to_be_bytes().into();
  ptr_ptr_buf.append(&mut ptr_len_buf);

  ptr_ptr_buf.as_ptr() as i32
}

#[no_mangle]
extern "C" fn on_request(input_ptr: i32) {
  let input_buf = read_from_memory(input_ptr);
  let input_str = String::from_utf8_lossy(input_buf.as_slice());
  dbg!(input_str);

  unsafe {
    let key_ptr = write_to_memory("X-Authentication".into());
    let value_ptr = get_metadata(key_ptr);
    let metadata_value =
      String::from_utf8(read_from_memory(value_ptr)).expect("Cant read value of key");
    dbg!(metadata_value);
  }

  unsafe {
    let ptr = write_to_memory("FROM WASM".into());
    send_response(ptr);
  };
}
