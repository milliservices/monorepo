
function read_from_memory(ptr: i32): u8[] {
  const data_ptr = load<i32>(ptr);
  // const data_len = load<u32>(ptr + sizeof<i32>(), sizeof<u32>());
  const data_len: u32 = 4;
  const data: u8[] = [];
  console.log(`${data_ptr} ${data_len}`)
  for (let i: u32 = 0; i < data_len; i++) {
    data.push(load<u8>(data_ptr + i, 1))
  }
  return data
}

export function on_request(input_ptr: i32): void {
  const ptr: u32 = 1;
  store<i32>(ptr, -5, sizeof<i32>());
  // const data: u8[] = [];
  // for (let i: u32 = 0; i < 4; i++) {
  //   data.push(load<u8>(ptr + i + 3, 1))
  // }
  console.log(`${load<ArrayBuffer>(ptr, 4)}`)


  // const byutes = read_from_memory(input_ptr);
  // let input_data = changetype<string>(byutes);
  // console.log(`:${byutes.join(', ')}:`);
}
