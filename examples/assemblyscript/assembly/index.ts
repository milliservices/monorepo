import { readAsString } from '@milliservices/assemblyscript/assembly'
import { ServiceCall, request, response } from '@milliservices/assemblyscript/assembly/service'

export function on_request(input_ptr: i32): void {
  const inputData = readAsString(input_ptr)
  console.log(`:: [ASS] input = ${inputData}`)

  console.log(`:: [ASS] Auth = ${request.metadata('authentication')}`)

  const handle = new ServiceCall("/rust-final");
  handle.setData(String.UTF8.encode("FOOBARITY BOEY"));
  handle.setMetadata("authentication", "other auth key");
  handle.execute();

  console.log(`:: [ASS] call response. Server: ${handle.response.metadata("Server")}; ${String.UTF8.decode(handle.response.data())}`);

  response.sendString('This is an ass response')
  response.sendString('. - From ass')
}
