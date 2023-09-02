use anyhow::Result;
use futures::Future;
use std::{collections::HashMap, sync::Arc};
use tokio::{
  sync::Mutex,
  task::{JoinError, JoinHandle},
};
use wasmtime::*;

use crate::{
  service::{ModuleConfig, ServiceInstance, ServiceModule},
  store::RecvMsg,
};

#[derive(Default)]
pub struct Node {
  modules: HashMap<String, ServiceModule>,
  module_config: HashMap<String, ModuleConfig>,
}

impl Node {
  pub fn new() -> Self {
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
      let module = ServiceModule::new(&cfg).await?;
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
}
