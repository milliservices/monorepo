import { readAsString, sendStringResponse } from '@milliservices/assemblyscript/assembly'

export function simple_io(input_ptr: i32): void {
  const data = readAsString(input_ptr);
  sendStringResponse(`${data}. adds output data`)
}
