use anyhow::Result;
use futures::Future;
use std::{collections::HashMap, ops::DerefMut, sync::Arc};
use tokio::{
  sync::Mutex,
  task::{JoinError, JoinHandle},
};
use wasmtime::*;

use crate::{
  service::{ModuleConfig, ServiceInstance, ServiceModule},
  store::{HostChannel, RecvMsg},
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
  pub fn new_ref() -> Arc<Mutex<Self>> {
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

pub async fn create_instance(node: Arc<Mutex<Node>>, name: String) -> Result<ServiceInstance> {
  node.lock().await.create_instance(name).await
}

pub async fn launch_node_msg_handler(
  node: Arc<Mutex<Node>>,
) -> futures::future::JoinAll<impl Future<Output = std::result::Result<Result<()>, JoinError>>> {
  let mut handles = vec![];

  for module in node.lock().await.modules.values() {
    let host_channel = module.host_channel.clone();
    let node_ref = Arc::clone(&node);

    let fut: JoinHandle<Result<()>> = tokio::task::spawn(async move {
      let mut ch = host_channel.lock().await;
      let (tx, rx) = ch.deref_mut();

      loop {
        if let Some(msg) = rx.recv().await {
          println!("[PRE] {}", &msg.name);
          let mut instance = create_instance(Arc::clone(&node_ref), msg.name.to_owned()).await?;
          println!("[POST] {}", &msg.name);
          instance.invoke(msg.data).await?;

          tx.send(RecvMsg {
            data: instance.get_response_data().to_owned(),
          })
          .await?;
        }
      }
    });

    handles.push(fut);
  }

  futures::future::join_all(handles)
}
