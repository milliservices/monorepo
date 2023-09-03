use std::mem::size_of;

extern "C" {
  pub fn send_response(data_ptr: i32);
  pub fn get_metadata(key_ptr: i32) -> i32;
  pub fn set_response_metadata(key_ptr: i32, value_ptr: i32);
  pub fn call_service(name_ptr: i32, data_ptr: i32) -> i32;

  // fn create_call(name_ptr: i32) -> u32;
  // fn set_data(id: u32, data_ptr: i32);
  // fn set_metdata(id: u32, key_ptr: i32, value_ptr: i32);
  // fn execute_call(id: u32);
  // fn get_response_metdata(id: u32, data_ptr: i32);
  // fn get_response_data(id: u32, data_ptr: i32);
}

pub fn read_as_string(ptr: i32) -> Result<String, std::string::FromUtf8Error> {
  let data = read_from_memory(ptr);
  String::from_utf8(data)
}

pub fn send_string_response(str: String) {
  let ptr = write_to_memory(str.into());
  unsafe {
    send_response(ptr);
  };
}

pub fn read_from_memory(ptr: i32) -> Vec<u8> {
  let (data_ptr, data_len) = get_mem_representation(ptr);
  let data_buf = unsafe { Vec::from_raw_parts(data_ptr as *mut u8, data_len, data_len) };
  let owned_data_buf = data_buf.to_owned();
  std::mem::forget(data_buf);

  owned_data_buf
}

pub fn write_to_memory(data: Vec<u8>) -> i32 {
  // let data = data.to_owned();
  let data_len = data.len();
  let data_ptr = data.as_ptr() as i32;

  let mut ptr_ptr_buf: Vec<u8> = data_ptr.to_le_bytes().into();
  let mut ptr_len_buf: Vec<u8> = (data_len as u32).to_le_bytes().into();
  ptr_ptr_buf.append(&mut ptr_len_buf);

  let ptr_ptr = ptr_ptr_buf.as_ptr() as i32;

  std::mem::forget(data);
  std::mem::forget(ptr_ptr_buf);

  ptr_ptr
}

fn get_mem_representation(ptr: i32) -> (i32, usize) {
  let mut buf = [0u8; size_of::<u32>()];

  let data_ptr_buf = unsafe { std::slice::from_raw_parts(ptr as *mut u8, size_of::<i32>()) };
  buf.copy_from_slice(data_ptr_buf);
  let data_ptr = i32::from_le_bytes(buf);

  let data_len_buf = unsafe { std::slice::from_raw_parts((ptr + 4) as *mut u8, size_of::<u32>()) };
  buf.copy_from_slice(data_len_buf);
  let data_len = u32::from_le_bytes(buf);

  (data_ptr, data_len as usize)
}
