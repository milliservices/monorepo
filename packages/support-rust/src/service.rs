use crate::internal;
use crate::memory;

pub mod request {
  use crate::internal;
  use crate::memory;

  pub fn get_metadata(key: &str) -> String {
    let value_ptr = unsafe { internal::get_metadata(memory::write_to_memory(key.into())) };
    memory::read_as_string(value_ptr).expect("Cant read value of key")
  }
}

pub mod response {
  use crate::internal;
  use crate::memory;

  pub fn send_data(data: Vec<u8>) {
    let ptr = memory::write_to_memory(data);
    unsafe { internal::send_response(ptr) }
  }

  pub fn send_string(data: &str) {
    send_data(data.into())
  }

  pub fn set_metadata(key: &str, value: &str) {
    let key_ptr = memory::write_to_memory(key.into());
    let value_ptr = memory::write_to_memory(value.into());
    unsafe { internal::set_response_metadata(key_ptr, value_ptr) }
  }
}

pub struct ServiceCall {
  id: u32,
  pub response: ServiceResponse,
}

impl ServiceCall {
  pub fn new(key: &str) -> Self {
    unsafe {
      let id = internal::service_new_request(memory::write_to_memory(key.into()));
      Self {
        id,
        response: ServiceResponse(id),
      }
    }
  }

  pub fn set_data(&self, data: Vec<u8>) {
    unsafe { internal::service_write_data(self.id, memory::write_to_memory(data)) }
  }

  pub fn set_metadata(&self, key: &str, value: &str) {
    unsafe {
      internal::service_set_metadata(
        self.id,
        memory::write_to_memory(key.into()),
        memory::write_to_memory(value.into()),
      )
    }
  }

  pub fn execute(&self) {
    unsafe {
      internal::service_execute(self.id);
    }
  }
}

pub struct ServiceResponse(u32);

impl ServiceResponse {
  pub fn new(id: u32) -> Self {
    Self(id)
  }

  pub fn data(&self) -> Vec<u8> {
    unsafe { memory::read_from_memory(internal::service_get_response(self.0)) }
  }

  pub fn metadata(&self, key: &str) -> String {
    unsafe {
      let key_ptr = memory::write_to_memory(key.into());
      memory::read_as_string(internal::service_get_response_metadata(self.0, key_ptr))
        .unwrap_or("".to_string())
    }
  }
}
