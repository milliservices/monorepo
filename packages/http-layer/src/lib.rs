use hyper::{server::conn::http1, service::Service, Request, Response, StatusCode};
use milliservices_core::{node::NodeRef, *};
use std::{collections::HashMap, future::Future, net::SocketAddr, pin::Pin, sync::Arc};
use tokio::net::TcpListener;

pub struct HttpLayer {
  node: NodeRef,
}

impl HttpLayer {
  pub fn new(node: NodeRef) -> Self {
    HttpLayer { node }
  }

  pub async fn init(&self, addr: SocketAddr) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
      let (stream, _) = listener.accept().await?;

      let node = Arc::clone(&self.node);
      tokio::task::spawn(async move {
        if let Err(err) = http1::Builder::new()
          .serve_connection(stream, HttpLayerService { node })
          .await
        {
          println!("Failed to serve connection: {:?}", err);
        }
      });
    }
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

  fn call(&mut self, request: Request<hyper::body::Body>) -> Self::Future {
    let node = Arc::clone(&self.node);
    let path = request.uri().path().to_string();

    let module_name = path; // TODO: remove trailing /
    let mut metadata = HashMap::from([
      ("@path".into(), request.uri().path().to_string()),
      ("@method".into(), request.method().to_string()),
    ]);

    for (name, value) in request.headers() {
      let key = name.to_string().to_lowercase();
      metadata.insert(key, value.to_str().unwrap_or("").to_string());
    }

    let body = request.into_body();

    Box::pin(async move {
      let data = hyper::body::to_bytes(body).await?;
      let instance = node::spawn_instance(node, module_name).await?;
      if let Some(mut instance) = instance {
        instance.update_metadata(metadata);
        instance.initialize().await?;
        instance.invoke(data.into()).await?;

        let mut builder = Response::builder().status(StatusCode::OK);

        for (k, v) in instance.get_response_metadata() {
          match &k[..] {
            "@status" => {
              let status = v.parse::<u16>().unwrap_or(200);
              builder = builder
                .status(StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR));
            }
            header_name => {
              builder = builder.header(header_name, v);
            }
          }
        }

        let response = builder.body(instance.get_response_data().to_owned().into())?;
        Ok(response)
      } else {
        Ok(
          Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("404. Not found".into())?,
        )
      }
    })
  }
}
