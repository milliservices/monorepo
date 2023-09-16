use crate::store::{SendMsg, ServiceRequest, ServiceStore};
use wasmtime::*;

fn get_memory(caller: &mut Caller<'_, ServiceStore>) -> Result<Memory> {
  if let Some(Extern::Memory(memory)) = caller.get_export("memory") {
    Ok(memory)
  } else {
    Err(Error::msg("Memory export not defined"))
  }
}

pub async fn call_service(
  mut caller: Caller<'_, ServiceStore>,
  name_ptr: i32,
  data_ptr: i32,
) -> Result<i32> {
  let memory = get_memory(&mut caller)?;

  let name = ServiceStore::read_string_from_memory(&caller.as_context(), memory, name_ptr)?;
  let data = ServiceStore::read_from_memory(&caller.as_context(), memory, data_ptr)?;

  let handler = caller.data_mut().handle_call_service.as_ref();
  if let Some(handler) = handler {
    let response = handler(SendMsg {
      name,
      data,
      ..Default::default()
    })
    .await?;
    let resp_ptr =
      ServiceStore::write_to_memory(&mut caller.as_context_mut(), memory, response.data)?;
    Ok(resp_ptr)
  } else {
    Err(Error::msg("Service call not supported for current module"))
  }
}

pub fn send_response(mut caller: Caller<'_, ServiceStore>, ptr: i32) -> Result<()> {
  let memory = get_memory(&mut caller)?;
  let mut buffer = ServiceStore::read_from_memory(&caller.as_context(), memory, ptr)?;
  caller.data_mut().response_data.append(&mut buffer);
  Ok(())
}

pub fn set_response_metadata(
  mut caller: Caller<'_, ServiceStore>,
  key_ptr: i32,
  value_ptr: i32,
) -> Result<()> {
  let memory = get_memory(&mut caller)?;

  let key = ServiceStore::read_string_from_memory(&caller.as_context(), memory, key_ptr)?;
  let value = ServiceStore::read_string_from_memory(&caller.as_context(), memory, value_ptr)?;

  caller.data_mut().response_metadata.insert(key, value);

  Ok(())
}

pub fn get_metadata(mut caller: Caller<'_, ServiceStore>, ptr: i32) -> Result<i32> {
  let memory = get_memory(&mut caller)?;

  let key = ServiceStore::read_string_from_memory(&caller.as_context(), memory, ptr)?;
  let metadata = &caller.data().metadata;

  let value: Vec<u8> = metadata
    .get(&key)
    .map(|s| s.to_owned())
    .unwrap_or_default()
    .into();

  let value_ptr = ServiceStore::write_to_memory(&mut caller.as_context_mut(), memory, value)?;

  Ok(value_ptr)
}

pub fn service_new_request(mut caller: Caller<'_, ServiceStore>, key_ptr: i32) -> Result<u32> {
  let memory = get_memory(&mut caller)?;
  let name = ServiceStore::read_string_from_memory(&caller.as_context(), memory, key_ptr)?;

  let req = ServiceRequest {
    name,
    ..Default::default()
  };

  let data = caller.data_mut();
  data.request_count += 1;
  data.requests.insert(data.request_count, req);

  Ok(data.request_count)
}

pub fn service_write_data(
  mut caller: Caller<'_, ServiceStore>,
  req_id: u32,
  data_ptr: i32,
) -> Result<()> {
  let memory = get_memory(&mut caller)?;
  let data = ServiceStore::read_from_memory(&caller.as_context(), memory, data_ptr)?;
  let store_data = caller.data_mut();
  if let Some(req) = store_data.requests.get_mut(&req_id) {
    req.data = data;
  }
  Ok(())
}

pub fn service_get_response(mut caller: Caller<'_, ServiceStore>, req_id: u32) -> Result<i32> {
  let memory = get_memory(&mut caller)?;
  let req = caller
    .data()
    .requests
    .get(&req_id)
    .ok_or(Error::msg("Request doesn't exist"))?
    .response_data
    .to_owned();
  let ptr = ServiceStore::write_to_memory(&mut caller.as_context_mut(), memory, req)?;
  Ok(ptr)
}

pub fn service_set_metadata(
  mut caller: Caller<'_, ServiceStore>,
  req_id: u32,
  key_ptr: i32,
  data_ptr: i32,
) -> Result<()> {
  let memory = get_memory(&mut caller)?;
  let key = ServiceStore::read_string_from_memory(&caller.as_context(), memory, key_ptr)?;
  let data = ServiceStore::read_string_from_memory(&caller.as_context(), memory, data_ptr)?;
  if let Some(req) = caller.data_mut().requests.get_mut(&req_id) {
    req.metadata.insert(key, data);
  }
  Ok(())
}

pub fn service_get_response_metadata(
  mut caller: Caller<'_, ServiceStore>,
  req_id: u32,
  key_ptr: i32,
) -> Result<i32> {
  let memory = get_memory(&mut caller)?;
  let key = ServiceStore::read_string_from_memory(&caller.as_context(), memory, key_ptr)?;
  let req = caller
    .data()
    .requests
    .get(&req_id)
    .ok_or(Error::msg("Request doesn't exist"))?
    .response_metadata
    .get(&key)
    .map_or("".to_string(), |s| s.to_owned());
  let ptr = ServiceStore::write_to_memory(&mut caller.as_context_mut(), memory, req.into())?;
  Ok(ptr)
}
pub async fn service_execute(mut caller: Caller<'_, ServiceStore>, req_id: u32) -> Result<()> {
  let data = caller.data_mut();

  if let Some(req) = data.requests.get_mut(&req_id) {
    req.executed = true;

    if let Some(handle) = data.handle_call_service.as_ref() {
      let response = handle(SendMsg {
        name: req.name.to_owned(),
        metadata: req.metadata.to_owned(),
        data: req.data.to_owned(),
      })
      .await?;
      req.response_metadata = response.metadata;
      req.response_data = response.data;
    }
  }

  Ok(())
}
