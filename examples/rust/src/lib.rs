use milliservices_utils::*;

#[no_mangle]
extern "C" fn on_request(input_ptr: i32) {
  let input_str = read_as_string(input_ptr).expect("aaa cant read input");
  println!(":: [RUST] input = {}", input_str);

  unsafe {
    let value_ptr = get_metadata(write_to_memory("X-Authentication".into()));
    let metadata_value = read_as_string(value_ptr).expect("Cant read value of key");
    dbg!(metadata_value);
  }

  unsafe {
    let key_ptr = write_to_memory("Content-Type".into());
    let value_ptr = write_to_memory("application/json".into());
    set_response_metadata(key_ptr, value_ptr);

    let key_ptr = write_to_memory("Server".into());
    let value_ptr = write_to_memory("milliservices_rust".into());
    set_response_metadata(key_ptr, value_ptr);

    let res = call_service(
      write_to_memory("/ass".into()),
      write_to_memory("Data sent to foobar".into()),
    );
    let data_buf = read_from_memory(res);
    let data = String::from_utf8_lossy(data_buf.as_slice());
    println!(":: [RUST] call response = {}", data);

    let res = call_service(
      write_to_memory("/ass".into()),
      write_to_memory("Another piece of ass data".into()),
    );
    let data_buf = read_from_memory(res);
    let data = String::from_utf8_lossy(data_buf.as_slice());
    println!(":: [RUST] call response 2 = {}", data);
  }

  send_string_response("Response is coming".into());
  send_string_response(" again".into());
  send_string_response(" and again".into());
  send_string_response(". So much data".into());
}

#[no_mangle]
extern "C" fn final_call(input_ptr: i32) {
  let input_str = read_as_string(input_ptr).expect("read input err");
  println!(":: [RUST FINAL] input = {}", input_str);

  unsafe {
    let value_ptr = get_metadata(write_to_memory("authentication".into()));
    let metadata_value = read_as_string(value_ptr).expect("Cant read value of key");
    dbg!(metadata_value);
  }

  unsafe {
    let key_ptr = write_to_memory("Server".into());
    let value_ptr = write_to_memory("milliservices_rust".into());
    set_response_metadata(key_ptr, value_ptr);

    let key_ptr = write_to_memory("@status".into());
    let value_ptr = write_to_memory("202".into());
    set_response_metadata(key_ptr, value_ptr);
  }
  send_string_response(format!("Final response. With input {input_str}"));
}
