use crate::internal;
use crate::memory;

pub struct ServiceRequest(u32);

impl ServiceRequest {
  pub fn new(key: &str) -> Self {
    unsafe {
      let id = internal::service_new_request(memory::write_to_memory(key.into()));
      Self(id)
    }
  }

  pub fn set_body(&self, data: Vec<u8>) {
    unsafe { internal::service_write_data(self.0, memory::write_to_memory(data)) }
  }

  pub fn set_metadata(&self, key: &str, value: &str) {
    unsafe {
      internal::service_set_metadata(
        self.0,
        memory::write_to_memory(key.into()),
        memory::write_to_memory(value.into()),
      )
    }
  }

  pub fn execute(&self) -> ServiceResponse {
    unsafe {
      internal::service_execute(self.0);
    }
    ServiceResponse::new(self.0)
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
