use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server as HyperServer, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::oneshot;

use crate::config::Config;
use crate::router::{Router, RouterRequest, Message, Tool};
use crate::provider::ProviderClient;

pub struct Server {
    config: Config,
    router: Router,
    provider_client: ProviderClient,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        let router = Router::new(config.clone());
        let provider_client = ProviderClient::new();
        Self {
            config,
            router,
            provider_client,
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let host = self.config.host.as_deref().unwrap_or("0.0.0.0:8080");
        let addr: SocketAddr = host.parse()?;

        let config = self.config.clone();
        let router = self.router.clone();
        let provider_client = self.provider_client.clone();

        let make_svc = make_service_fn(move |_conn| {
            let config = config.clone();
            let router = router.clone();
            let provider_client = provider_client.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let config = config.clone();
                    let router = router.clone();
                    let provider_client = provider_client.clone();
                    handle_request(req, config, router, provider_client)
                }))
            }
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

    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = self.shutdown_tx.take() {
            tx.send(()).map_err(|_| "Shutdown signal failed")?;
            println!("🛑 Server is shutting down gracefully...");
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub system: Option<Value>, // Can be string or array
    #[serde(default)]
    pub tools: Option<Vec<Tool>>,
    #[serde(default)]
    pub thinking: Option<Value>,
    // Additional fields Claude Code sends
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub metadata: Option<Value>,
}


async fn handle_request(
    req: Request<Body>,
    config: Config,
    router: Router,
    provider_client: ProviderClient,
) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();
    let method = req.method();

    // Skip auth for health checks
    if !matches!((method, path), (&Method::GET, "/") | (&Method::GET, "/health")) {
        if let Err(resp) = check_auth(&req, &config) {
            return Ok(resp);
        }
    }

    match (method, path) {
        (&Method::GET, "/") | (&Method::GET, "/health") => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(Body::from("OK"))
                .unwrap())
        }
        (&Method::POST, "/v1/messages") => {
            handle_claude_request(req, router, provider_client, config).await
        }
        _ => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/plain")
                .body(Body::from("Not Found"))
                .unwrap())
        }
    }
}

fn check_auth(req: &Request<Body>, config: &Config) -> Result<(), Response<Body>> {
    if config.apikey.is_none() {
        return Ok(());
    }

    let api_key = config.apikey.as_ref().unwrap();
    
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .or_else(|| req.headers().get("x-api-key").and_then(|h| h.to_str().ok()));

    match auth_header {
        Some(key) if key == api_key => Ok(()),
        _ => Err(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("Content-Type", "text/plain")
            .body(Body::from("Unauthorized"))
            .unwrap()),
    }
}

async fn handle_claude_request(
    req: Request<Body>,
    router: Router,
    provider_client: ProviderClient,
    config: Config,
) -> Result<Response<Body>, Infallible> {
    let bytes = match hyper::body::to_bytes(req.into_body()).await {
        Ok(b) => b,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Invalid request body"}"#))
                .unwrap())
        }
    };

    // Debug: Log the incoming request body
    let body_str = String::from_utf8_lossy(&bytes);
    log::debug!("Incoming request body: {}", body_str);
    
    let claude_req: Result<ClaudeRequest, _> = serde_json::from_slice(&bytes);
    let claude_req = match claude_req {
        Ok(req) => req,
        Err(e) => {
            log::error!("❌ Failed to parse JSON: {} | Body: {}", e, body_str);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Invalid JSON"}"#))
                .unwrap());
        }
    };

    let router_request = RouterRequest {
        model: Some(claude_req.model.clone()),
        messages: claude_req.messages.clone(),
        system: claude_req.system.clone(), // Already a Value, no conversion needed
        tools: claude_req.tools.clone(),
        thinking: claude_req.thinking.clone().and_then(|v| v.as_bool()),
    };

    // Route the request
    let route = match router.route_request(&router_request) {
        Ok(route) => route,
        Err(e) => {
            log::error!("Routing error: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Routing failed"}"#))
                .unwrap());
        }
    };

    log::info!("🧭 Routing request to: {}", route);

    // Forward to provider
    match provider_client.send_claude_request(&route, &claude_req, &config).await {
        Ok(provider_response) => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(provider_response.to_string()))
                .unwrap())
        }
        Err(e) => {
            log::error!("Provider error: {}", e);
            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Provider request failed"}"#))
                .unwrap())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use hyper::Client;

    #[tokio::test]
    async fn test_server_health_check() {
        let config = Config {
            providers: vec![],
            router: crate::config::RouterConfig {
                default: "test".to_string(),
                background: None,
                think: None,
                long_context: None,
                web_search: None,
            },
            apikey: None,
            host: Some("127.0.0.1:0".to_string()),
            log: None,
        };
        let mut server = Server::new(config);
        
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let config = server.config.clone();
        let router = server.router.clone();
        
        let make_svc = make_service_fn(move |_conn| {
            let config = config.clone();
            let router = router.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let config = config.clone();
                    let router = router.clone();
                    handle_request(req, config, router)
                }))
            }
        });
        
        let server = HyperServer::bind(&addr).serve(make_svc);
        let addr = server.local_addr();
        
        tokio::spawn(async move {
            let _ = server.await;
        });
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let client = Client::new();
        let uri: hyper::Uri = format!("http://{}/health", addr).parse().unwrap();
        let resp = client.get(uri).await.unwrap();
        
        assert_eq!(resp.status(), 200);
    }
}