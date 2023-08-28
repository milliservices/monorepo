extern "C" {
  fn send_response(input_ptr: i32, input_size: u32);
}

#[no_mangle]
extern "C" fn on_request(input_ptr: i32, input_size: u32) {
  let ptr = input_ptr as *mut u8;
  let data = unsafe { Vec::from_raw_parts(ptr, input_size as usize, input_size as usize) };
  println!("{:?}", data);
  std::mem::forget(data);

  unsafe {
    send_response(0, 5);
  }
}
