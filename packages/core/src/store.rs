use anyhow::Result;
use std::{collections::HashMap, mem::size_of, sync::Arc};
use wasmtime::{Memory, StoreContext, StoreContextMut};
use wasmtime_wasi::WasiCtx;

#[derive(Debug, Clone)]
pub struct SendMsg {
  pub name: String,
  pub data: Vec<u8>,
}
#[derive(Debug, Clone)]
pub struct RecvMsg {
  pub data: Vec<u8>,
}

pub type HandleCallService = Arc<
  dyn Send
    + Sync
    // + 'static
    + Fn(
      SendMsg,
    ) -> std::pin::Pin<
      Box<
        dyn std::future::Future<Output = Result<RecvMsg>>
          // + 'static
          + Send,
      >,
    >,
>;

pub struct ServiceStore {
  pub metadata: HashMap<String, String>,
  pub pointer_offset: u32,
  pub response_data: Vec<u8>,
  pub response_metadata: HashMap<String, String>,
  pub wasi_ctx: WasiCtx,
  pub handle_call_service: Option<HandleCallService>,
}

impl ServiceStore {
  fn mem_write(
    store: &mut StoreContextMut<ServiceStore>,
    memory: Memory,
    data: Vec<u8>,
  ) -> Result<u32> {
    let store_data = store.data_mut();
    let data_ptr = store_data.pointer_offset;

    // TODO: Cyclic increment back to 1 at end?
    store_data.pointer_offset += data.len() as u32;

    memory.write(store, data_ptr as usize, data.as_slice())?;

    Ok(data_ptr)
  }

  pub fn write_to_memory(
    store: &mut StoreContextMut<ServiceStore>,
    memory: Memory,
    data: Vec<u8>,
  ) -> Result<i32> {
    let data_len = data.len();

    // Write data to memory
    let data_ptr = Self::mem_write(store, memory, data)?;

    let mut ptr_ptr_buf: Vec<u8> = data_ptr.to_le_bytes().into();
    let mut ptr_len_buf: Vec<u8> = (data_len as u32).to_le_bytes().into();
    ptr_ptr_buf.append(&mut ptr_len_buf);

    let ptr = Self::mem_write(store, memory, ptr_ptr_buf)?;

    Ok(ptr as i32)
  }

  pub fn read_from_memory(
    store: &StoreContext<ServiceStore>,
    memory: Memory,
    ptr: i32,
  ) -> Result<Vec<u8>> {
    let mut buffer = [0u8; size_of::<i32>()];
    memory.read(store, ptr as usize, &mut buffer)?;
    let data_ptr = i32::from_le_bytes(buffer);

    let mut buffer = [0u8; size_of::<u32>()];
    memory.read(store, ptr as usize + size_of::<i32>(), &mut buffer)?;
    let data_len = u32::from_le_bytes(buffer);

    let mut buffer = vec![0u8; data_len as usize];
    memory.read(store, data_ptr as usize, &mut buffer)?;

    Ok(buffer)
  }

  pub fn read_string_from_memory(
    store: &StoreContext<ServiceStore>,
    memory: Memory,
    ptr: i32,
  ) -> Result<String> {
    let bytes = Self::read_from_memory(store, memory, ptr)?;
    let str = String::from_utf8(bytes)?;
    Ok(str)
  }
}
