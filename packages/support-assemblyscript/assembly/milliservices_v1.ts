
export declare function send_response(data_ptr: i32): void;
export declare function get_metadata(key_ptr: i32): i32;
export declare function set_response_metadata(key_ptr: i32, value_ptr: i32): void;

export declare function service_new_request(ptr: i32): u32;
export declare function service_execute(id: u32): void;
export declare function service_write_data(id: u32, ptr: i32): void;
export declare function service_get_response(id: u32): i32;
export declare function service_set_metadata(id: u32, key_ptr: i32, value_ptr: i32): void;
export declare function service_get_response_metadata(id: u32, key_ptr: i32): i32;
