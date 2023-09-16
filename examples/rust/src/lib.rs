use milliservices_support::memory;
use milliservices_support::service;

#[no_mangle]
extern "C" fn on_request(input_ptr: i32) {
  let input_str = memory::read_as_string(input_ptr).expect("aaa cant read input");
  println!(":: [RUST] input = {}", input_str);

  dbg!(service::request::get_metadata("authentication"));

  service::response::set_metadata("Content-Type", "text/plain");
  service::response::set_metadata("Server", "milliservices_rust_core");

  let handle = service::ServiceCall::new("/rust-final");
  handle.set_data("sdfhsdkfjhsdfsdf".into());
  handle.set_metadata("authentication", "foobario");
  handle.execute();
  println!(
    ":: [RUST] call response rust-final = Server: {}; {}",
    handle.response.metadata("Server"),
    String::from_utf8_lossy(&handle.response.data())
  );

  service::response::set_metadata("@status", &handle.response.metadata("@status"));

  let handle = service::ServiceCall::new("/ass");
  handle.set_data("SOME DAA TO ASS".into());
  handle.execute();
  println!(
    ":: [RUST] call response ass = {}",
    String::from_utf8_lossy(&handle.response.data())
  );

  let handle = service::ServiceCall::new("/haskell");
  handle.set_data("Hask me for some data and ye shall receive".into());
  handle.execute();
  println!(
    ":: [RUST] call response haskell = {}",
    String::from_utf8_lossy(&handle.response.data())
  );

  service::response::send_string("Response is coming");
  service::response::send_string(" again");
  service::response::send_string(" and again");
  service::response::send_string(". So much data");
}

#[no_mangle]
extern "C" fn final_call(input_ptr: i32) {
  let input_str = memory::read_as_string(input_ptr).expect("read input err");
  println!(":: [RUST FINAL] input = {}", input_str);

  dbg!(service::request::get_metadata("authentication"));

  service::response::set_metadata("@status", "202");
  service::response::set_metadata("Server", "milliservices_rust_final");

  service::response::send_string(&format!("Final response. With input {input_str}"));
}
