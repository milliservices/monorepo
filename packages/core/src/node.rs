use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use wasmtime::*;

use crate::{
  service::{ModuleConfig, ServiceInstance, ServiceModule},
  store::{HostChannel, ModuleChannel, RecvMsg, SendMsg},
};

pub struct Node {
  modules: HashMap<String, ServiceModule>,
  module_config: HashMap<String, ModuleConfig>,
  pub host_channel: HostChannel,
  pub module_channel: ModuleChannel,
}

impl Node {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    let (send_1, recv_1) = tokio::sync::mpsc::channel::<SendMsg>(10);
    let (send_2, recv_2) = tokio::sync::mpsc::channel::<RecvMsg>(10);
    // TODO: Use different channels per instances
    Self {
      modules: HashMap::new(),
      module_config: HashMap::new(),
      host_channel: Arc::new(Mutex::new((send_2, recv_1))),
      module_channel: Arc::new(Mutex::new((send_1, recv_2))),
    }
  }

  pub async fn load_module(&mut self, cfg: ModuleConfig) -> Result<&ServiceModule> {
    let path = cfg.path.to_owned();
    let name = cfg.name.to_owned();

    let modules = &mut self.modules;
    let module_config = &mut self.module_config;

    if !modules.contains_key(&path) {
      let module = ServiceModule::new(&path, self.module_channel.clone()).await?;
      modules.insert(path.to_owned(), module);
      module_config.insert(name, cfg);
    }
    Ok(modules.get(&path).expect("unreachable: load_module"))
  }

  pub async fn create_instance(&mut self, name: String) -> Result<ServiceInstance> {
    let cfg = self
      .module_config
      .get(&name)
      .ok_or(Error::msg("Module not loaded"))?;
    let symbol = cfg.symbol.to_owned();
    let module = self.load_module(cfg.to_owned()).await?;
    module.instantiate(&symbol).await
  }

  #[allow(clippy::await_holding_lock)]
  pub fn launch_handler(&mut self) -> tokio::task::JoinHandle<Result<()>> {
    let channel_mutex = self.host_channel.clone();

    tokio::task::spawn(async move {
      loop {
        let mut channel = channel_mutex.lock().await;
        if let Some(msg) = channel.1.recv().await {
          // TODO: call the {msg.name} module
          dbg!(msg);
          channel
            .0
            .send(RecvMsg {
              data: "This is response to req".into(),
            })
            .await?;
        }
      }
    })
  }
}
