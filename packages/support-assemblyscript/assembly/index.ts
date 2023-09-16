import { writeToMemory } from './memory';
import * as internal from './milliservices_v1'

export * from './memory';

export function sendResponse(buffer: ArrayBuffer): void {
  const ptr = writeToMemory(buffer);
  internal.send_response(ptr);
}

export function sendStringResponse(response: string): void {
  sendResponse(String.UTF8.encode(response));
}
