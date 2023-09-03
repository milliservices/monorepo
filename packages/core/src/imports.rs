use crate::store::{SendMsg, ServiceStore};
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
    let response = handler(SendMsg { name, data }).await?;
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
