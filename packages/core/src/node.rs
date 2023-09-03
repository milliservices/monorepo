use anyhow::Result;

use futures::Future;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use wasmtime::*;

use crate::{
  service::{ModuleConfig, ServiceInstance, ServiceModule},
  store::{HandleCallService, RecvMsg},
};

#[derive(Default)]
pub struct Node {
  modules: HashMap<String, ServiceModule>,
  module_config: HashMap<String, ModuleConfig>,
}

pub type NodeRef = Arc<Mutex<Node>>;

impl Node {
  pub fn new() -> Self {
    Self {
      modules: HashMap::new(),
      module_config: HashMap::new(),
    }
  }
  pub fn new_ref() -> NodeRef {
    Arc::new(Mutex::new(Self::new()))
  }

  pub async fn load_module(&mut self, cfg: ModuleConfig) -> Result<&ServiceModule> {
    let path = cfg.path.to_owned();
    let name = cfg.name.to_owned();

    let modules = &mut self.modules;
    let module_config = &mut self.module_config;

    if !modules.contains_key(&path) {
      let module = ServiceModule::new(&cfg).await?;
      modules.insert(path.to_owned(), module);
    }
    module_config.entry(name).or_insert(cfg);

    Ok(modules.get(&path).expect("unreachable: load_module"))
  }

  pub async fn create_instance(&mut self, name: String) -> Result<ServiceInstance> {
    let cfg = self
      .module_config
      .get(&name)
      .ok_or(Error::msg(format!("Module not loaded: \"{name}\"")))?;
    let symbol = cfg.symbol.to_owned();
    let module = self.load_module(cfg.to_owned()).await?;
    module.instantiate(&symbol).await
  }
}

pub fn spawn_instance(
  node: NodeRef,
  name: String,
) -> std::pin::Pin<Box<dyn Future<Output = Result<ServiceInstance>> + Send>> {
  Box::pin(async {
    // let _ = node.try_lock()?; // Force try? to get early errors on
    let mut instance = node.lock().await.create_instance(name).await?;

    let call_service: HandleCallService = Arc::new(move |msg| {
      let node_ref_cb = Arc::clone(&node);

      Box::pin(async move {
        let mut instance = spawn_instance(Arc::clone(&node_ref_cb), msg.name.to_owned()).await?;
        instance.invoke(msg.data).await?;

        Ok(RecvMsg {
          data: instance.get_response_data().to_owned(),
        })
      })
    });

    instance.set_call_service_handler(call_service).await;

    Ok(instance)
  })
}
