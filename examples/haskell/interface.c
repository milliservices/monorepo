#include <stdint.h>

void _send_response(int ptr)
    __attribute__((__import_module__("milliservices_v1"),
                   __import_name__("send_response")));
void sendResponse(int ptr) { return _send_response(ptr); }

void _set_response_metadata(int key_ptr, int value_ptr)
    __attribute__((__import_module__("milliservices_v1"),
                   __import_name__("set_response_metadata")));
void setResponseMetadata(int key_ptr, int value_ptr) {
  return _set_response_metadata(key_ptr, value_ptr);
}
