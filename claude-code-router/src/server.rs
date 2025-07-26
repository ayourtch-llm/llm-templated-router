use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server as HyperServer};
use tokio::sync::oneshot;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

pub struct Server {
    config: Config,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr: SocketAddr = self.config.addr().parse()?;

        let make_svc = make_service_fn(|_conn| async {
            Ok::<_, Infallible>(service_fn(router))
        });

        let (tx, rx) = oneshot::channel::<()>();
        self.shutdown_tx = Some(tx);

        let server = HyperServer::bind(&addr)
            .serve(make_svc)
            .with_graceful_shutdown(async {
                rx.await.ok();
            });

        println!("🚀 Server started on http://{}", addr);

        if let Err(e) = server.await {
            eprintln!("❌ Server error: {}", e);
            return Err(Box::new(e));
        }

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tx) = self.shutdown_tx.take() {
            tx.send(()).map_err(|_| "Shutdown signal failed")?;
            println!("🛑 Server is shutting down gracefully...");
        }
        Ok(())
    }
}

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::POST, "/") => {
            let response = Response::builder()
                .status(200)
                .header("Content-Type", "text/plain")
                .body(Body::from("OK"))
                .unwrap();
            Ok(response)
        }
        _ => {
            let response = Response::builder()
                .status(404)
                .header("Content-Type", "text/plain")
                .body(Body::from("Not Found"))
                .unwrap();
            Ok(response)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Client;

    #[tokio::test]
    async fn test_server_starts_and_responds() {
        let config = Config::new("127.0.0.1", 0);
        let mut server = Server::new(config);

        let config_clone = server.config.clone();
        tokio::spawn(async move {
            let _ = server.start().await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}