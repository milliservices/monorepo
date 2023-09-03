use milliservices_utils::*;

#[no_mangle]
extern "C" fn simple_io(input_ptr: i32) {
  let input_str = read_as_string(input_ptr).expect("aaa cant read input");
  send_string_response(format!("Getting output after {input_str}"));
}
