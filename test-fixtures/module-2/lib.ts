import { readAsString, readFromMemory, sendResponse, sendStringResponse } from '@milliservices/assemblyscript/assembly'

export function simple_io(input_ptr: i32): void {
  const data = readAsString(input_ptr);
  sendStringResponse(`${data}. adds output data`)
}

export function simple_calculations(input_ptr: i32): void {
  const buffer = readFromMemory(input_ptr);
  const view = new DataView(buffer);
  const num1 = view.getInt32(0);
  const num2 = view.getInt32(4);
  const result = num1 + num2;

  const resultView = new DataView(new ArrayBuffer(4));
  resultView.setInt32(0, result);
  sendResponse(resultView.buffer)
}
