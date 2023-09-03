import {
  callService,
  readAsString,
  sendStringResponse,
} from '@milliservices/assemblyscript/assembly'

export function on_request(input_ptr: i32): void {
  const inputData = readAsString(input_ptr)
  console.log(`:: [ASS] input = ${inputData}`)

  const serviceData = callService("rust-final", String.UTF8.encode("FOOBAR"));
  console.log(`:: [ASS] call response = ${String.UTF8.decode(serviceData)}`);

  sendStringResponse('This is an ass response')
}
