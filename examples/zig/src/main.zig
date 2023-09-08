const std = @import("std");
const testing = std.testing;

extern "env" fn send_response(value: i32) void;

export fn add(a: i32, b: i32) i32 {
    return a + b;
}

pub fn main() u8 {
    return 0;
}

