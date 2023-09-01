use anyhow::Result;
use futures::Future;
use std::collections::HashMap;
use tokio::task::{JoinError, JoinHandle};
use wasmtime::*;

use crate::{
  service::{ModuleConfig, ServiceInstance, ServiceModule},
  store::RecvMsg,
};

pub type TaskHandler =
  futures::future::JoinAll<impl Future<Output = std::result::Result<Result<()>, JoinError>>>;

pub struct Node {
  modules: HashMap<String, ServiceModule>,
  module_config: HashMap<String, ModuleConfig>,
}

impl Node {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    // TODO: Use different channels per instances
    Self {
      modules: HashMap::new(),
      module_config: HashMap::new(),
    }
  }

  pub async fn load_module(&mut self, cfg: ModuleConfig) -> Result<&ServiceModule> {
    let path = cfg.path.to_owned();
    let name = cfg.name.to_owned();

    let modules = &mut self.modules;
    let module_config = &mut self.module_config;

    if !modules.contains_key(&path) {
      let module = ServiceModule::new(&path).await?;
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
  pub fn launch_handler(&mut self) -> TaskHandler {
    let mut handles = vec![];

    for (_key, module) in self.modules.iter() {
      let channel_mutex = module.host_channel.clone();

      let fut: JoinHandle<Result<()>> = tokio::task::spawn(async move {
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
      });

      handles.push(fut);
    }

    futures::future::join_all(handles)
  }
}
