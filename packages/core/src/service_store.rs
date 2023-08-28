use anyhow::Result;
use std::{collections::HashMap, mem::size_of};
use wasmtime::{Memory, StoreContext, StoreContextMut};
use wasmtime_wasi::WasiCtx;

pub struct ServiceStore {
  pub wasi_ctx: WasiCtx,
  pub metadata: HashMap<String, String>,
  pub pointer_offset: u32,
}

impl ServiceStore {
  fn mem_write(
    store: &mut StoreContextMut<ServiceStore>,
    memory: Memory,
    data: Vec<u8>,
  ) -> Result<u32> {
    let store_data = store.data_mut();
    let data_ptr = store_data.pointer_offset;

    // TODO: Cyclic incremenet, maybe?
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

    let mut ptr_ptr_buf: Vec<u8> = data_ptr.to_be_bytes().into();
    let mut ptr_len_buf: Vec<u8> = (data_len as u32).to_be_bytes().into();
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
    let data_ptr = i32::from_be_bytes(buffer);

    let mut buffer = [0u8; size_of::<u32>()];
    memory.read(store, ptr as usize + size_of::<i32>(), &mut buffer)?;
    let data_len = u32::from_be_bytes(buffer);

    let mut buffer = vec![0u8; data_len as usize];
    memory.read(store, data_ptr as usize, &mut buffer)?;
    Ok(buffer)
  }
}
