use milliservices_support::*;

#[no_mangle]
extern "C" fn simple_io(input_ptr: i32) {
  let input_str = memory::read_as_string(input_ptr).expect("aaa cant read input");
  service::response::send_string(&format!("Getting output after {input_str}"));
}
