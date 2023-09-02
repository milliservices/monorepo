import { send_response, call_service } from './env'

function read_from_memory(ptr: i32): ArrayBuffer {
  const data_ptr = load<i32>(ptr);
  const data_len = load<u32>(ptr + sizeof<i32>());
  const data = new Uint8Array(data_len);
  for (let i: u32 = 0; i < data_len; i++) {
    data[i] = load<u8>(data_ptr + i);
  }
  return data.buffer
}

// Managed pointer
let last_memory_write = memory.data(8);

function write_to_memory(buffer: ArrayBuffer): i32 {
  const view = new DataView(buffer);
  // let ptr_ptr: usize = memory.data(sizeof<i32>() + sizeof<u32>());
  // let ptr: usize = memory.data(100);
  let ptr = last_memory_write;
  let ptr_ptr = ptr + buffer.byteLength;
  last_memory_write = ptr_ptr + sizeof<i32>() + sizeof<u32>();

  for (let i: i32 = 0; i < buffer.byteLength; i++) {
    store<u8>(ptr + i, view.getUint8(i));
  }

  store<i32>(ptr_ptr, ptr);
  store<u32>(ptr_ptr + sizeof<i32>(), buffer.byteLength);
  return ptr_ptr as i32
}

export function on_request(input_ptr: i32): void {
  const input_buf = read_from_memory(input_ptr);
  const input_data = String.UTF8.decode(input_buf);
  console.log(`:: [ASS] input = ${input_data}`);

  const service_key_ptr = write_to_memory(String.UTF8.encode("rust-final"));
  const value_ptr = write_to_memory(String.UTF8.encode("FOOBAR"));
  // console.log(`ptr = ${service_key_ptr}`);
  // console.log(`keyptr = ${load<i32>(service_key_ptr)}`);
  // console.log(`read from ass = ${String.UTF8.decode(read_from_memory(service_key_ptr))}`);
  const data_ptr = call_service(
    service_key_ptr,
    value_ptr,
  );
  console.log(`:: [ASS] call response = ${String.UTF8.decode(read_from_memory(data_ptr))}`);

  const resp_ptr = write_to_memory(String.UTF8.encode("This is an ass response"));
  send_response(resp_ptr);
}
