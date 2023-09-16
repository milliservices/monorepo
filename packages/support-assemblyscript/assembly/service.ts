import { readAsString, readFromMemory, writeToMemory } from './memory';
import * as internal from './milliservices_v1'

@final export class request {
  static metadata(key: string): string {
    const ptr = internal.get_metadata(writeToMemory(String.UTF8.encode(key)));
    return readAsString(ptr);
  }
}

@final export class response {
  static metadata(key: string, value: string): void {
    const key_ptr = writeToMemory(String.UTF8.encode(key));
    const value_ptr = writeToMemory(String.UTF8.encode(value));
    internal.set_response_metadata(key_ptr, value_ptr);
  }

  static sendData(data: ArrayBuffer): void {
    internal.send_response(writeToMemory(data));
  }

  static sendString(data: string): void {
    response.sendData(String.UTF8.encode(data));
  }
}

@final export class ServiceCall {
  private id: u32;
  public response: ServiceResponse;

  constructor(name: string) {
    const key_ptr = writeToMemory(String.UTF8.encode(name));
    const id = internal.service_new_request(key_ptr);
    this.id = id;
    this.response = new ServiceResponse(id);
  }

  setMetadata(key: string, value: string): void {
    const key_ptr = writeToMemory(String.UTF8.encode(key));
    const value_ptr = writeToMemory(String.UTF8.encode(value));
    internal.service_set_metadata(this.id, key_ptr, value_ptr);
  }

  setData(data: ArrayBuffer): void {
    internal.service_write_data(this.id, writeToMemory(data));
  }

  execute(): void {
    internal.service_execute(this.id);
  }
}

@final class ServiceResponse {
  private id: u32;

  constructor(id: u32) {
    this.id = id
  }

  metadata(key: string): string {
    const key_ptr = writeToMemory(String.UTF8.encode(key));
    const val_ptr = internal.service_get_response_metadata(this.id, key_ptr);
    return readAsString(val_ptr);
  }

  data(): ArrayBuffer {
    const ptr = internal.service_get_response(this.id);
    return readFromMemory(ptr);
  }
}

