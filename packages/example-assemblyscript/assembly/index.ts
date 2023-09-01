import { send_response } from './env'

function read_from_memory(ptr: i32): Uint8Array {
  const data_ptr = load<i32>(ptr);
  const data_len = load<u32>(ptr + sizeof<i32>());
  const data = new Uint8Array(data_len);
  for (let i: u32 = 0; i < data_len; i++) {
    data[i] = load<u8>(data_ptr + i);
  }
  return data
}

function write_to_memory(data: ArrayBuffer): i32 {
  const view = new DataView(data);
  let ptr: usize = memory.data(1);
  for (let i: i32 = 0; i < data.byteLength; i++) {
    store<u8>(ptr + i, view.getUint8(i));
  }

  const ptr_ptr = ptr + data.byteLength;
  store<i32>(ptr_ptr, ptr);
  store<u32>(ptr_ptr + sizeof<i32>(), data.byteLength);
  return ptr_ptr as i32
}

export function on_request(input_ptr: i32): void {
  const input_buf = read_from_memory(input_ptr);
  const input_data = String.UTF8.decode(input_buf.buffer);
  console.log(input_data);

  const resp_ptr = write_to_memory(String.UTF8.encode("Hello world"));
  send_response(resp_ptr);
}
