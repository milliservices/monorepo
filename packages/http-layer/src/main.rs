use hyper::{server::conn::http1, service::Service, Request, Response, StatusCode};
use std::{future::Future, net::SocketAddr, pin::Pin, sync::Arc, collections::HashMap};
use tokio::net::TcpListener;
use milliservices_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

  let listener = TcpListener::bind(addr).await?;
  println!("Listening on http://{}", addr);

  let node = node::Node::new_ref();

  let module_configs = vec![
    service::ModuleConfig {
      name: "rust".to_string(),
      path: "./target/wasm32-wasi/debug/example_rust_wasm.wasm".to_string(),
      symbol: "on_request".to_string(),
      ..Default::default()
    },
    service::ModuleConfig {
      name: "ass".to_string(),
      path: "./examples/assemblyscript/build/debug.wasm".to_string(),
      symbol: "on_request".to_string(),
      ..Default::default()
    },
    service::ModuleConfig {
      name: "rust-final".to_string(),
      path: "./target/wasm32-wasi/debug/example_rust_wasm.wasm".to_string(),
      symbol: "final_call".to_string(),
      ..Default::default()
    },
  ];

  for cfg in module_configs {
    node.lock().await.load_module(cfg).await?;
  }

  loop {
    let (stream, _) = listener.accept().await?;
    // let io = TokioIo::new(stream);

    let node = Arc::clone(&node);
    tokio::task::spawn(async move {
      if let Err(err) = http1::Builder::new()
        .serve_connection(
          stream,
          HttpLayerService { node },
        )
        .await
      {
        println!("Failed to serve connection: {:?}", err);
      }
    });
  }
}

struct HttpLayerService {
  node: node::NodeRef,
}

impl Service<Request<hyper::body::Body>> for HttpLayerService {
  type Response = Response<hyper::body::Body>;
  type Error = anyhow::Error;
  type Future = Pin<Box<dyn Future<Output = anyhow::Result<Self::Response>> + Send>>;

  fn poll_ready(
    &mut self,
    _cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), Self::Error>> {
    std::task::Poll::Ready(Ok(()))
  }

  fn call(&mut self, req: Request<hyper::body::Body>) -> Self::Future {
    let node = Arc::clone(&self.node);
    let uri = req.uri().path().to_string();

    let metadata = HashMap::from([
                ("@method".into(), req.uri().path().to_string()),
                ("@path".into(), req.method().to_string()),
               // TODO: add headers
            ]);

    Box::pin(async move {
      println!("{}", uri);

      let mut instance = node::spawn_instance(node, "rust".into()).await?;
            instance.update_metadata(metadata);
      instance.invoke("Request data incoming".into()).await?;

      // TODO: Set response metadata

      let builder = Response::builder()
        .header("Foo", "Bar")
        .status(StatusCode::OK)
        .body(instance.get_response_data().to_owned().into())
        .expect("unable to build response");

      Ok(builder)
    })
  }
}
