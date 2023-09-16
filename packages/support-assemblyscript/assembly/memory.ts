export function readAsString(ptr: i32): string {
  const buffer = readFromMemory(ptr);
  return String.UTF8.decode(buffer);
}

export function readFromMemory(ptr: i32): ArrayBuffer {
  const data_ptr = load<i32>(ptr);
  const data_len = load<u32>(ptr + sizeof<i32>());
  const data = new Uint8Array(data_len);
  for (let i: u32 = 0; i < data_len; i++) {
    data[i] = load<u8>(data_ptr + i);
  }
  return data.buffer
}

// Managed pointer offset
let current_pointer_offset = memory.data(8);

export function writeToMemory(buffer: ArrayBuffer): i32 {
  const view = new DataView(buffer);

  let ptr = current_pointer_offset;
  let ptr_ptr = ptr + buffer.byteLength;
  current_pointer_offset = ptr_ptr + sizeof<i32>() + sizeof<u32>();

  for (let i: i32 = 0; i < buffer.byteLength; i++) {
    store<u8>(ptr + i, view.getUint8(i));
  }

  store<i32>(ptr_ptr, ptr);
  store<u32>(ptr_ptr + sizeof<i32>(), buffer.byteLength);
  return ptr_ptr as i32
}

