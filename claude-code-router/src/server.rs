use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server as HyperServer, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::oneshot;

use crate::config::Config;
use crate::router::{Router, RouterRequest, Message, Tool};

pub struct Server {
    config: Config,
    router: Router,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        let router = Router::new(config.clone());
        Self {
            config,
            router,
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let host = self.config.host.as_deref().unwrap_or("0.0.0.0:8080");
        let addr: SocketAddr = host.parse()?;

        let config = self.config.clone();
        let router = self.router.clone();

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
struct ClaudeRequest {
    model: String,
    messages: Vec<Message>,
    system: Option<String>,
    tools: Option<Vec<Tool>>,
    thinking: Option<Value>,
}

#[derive(Serialize)]
struct RoutingResponse {
    routed_to: String,
    token_count: u32,
}

async fn handle_request(
    req: Request<Body>,
    config: Config,
    router: Router,
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
            handle_claude_request(req, router).await
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

    let claude_req: Result<ClaudeRequest, _> = serde_json::from_slice(&bytes);
    let claude_req = match claude_req {
        Ok(req) => req,
        Err(e) => {
            eprintln!("❌ Failed to parse JSON: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Invalid JSON"}"#))
                .unwrap());
        }
    };

    let router_request = RouterRequest {
        model: Some(claude_req.model),
        messages: claude_req.messages,
        system: claude_req.system.map(|s| serde_json::Value::String(s)),
        tools: claude_req.tools,
        thinking: claude_req.thinking.and_then(|v| v.as_bool()),
    };

    let token_count = estimate_token_count(&router_request);
    match router.route_request(&router_request) {
        Ok(route) => {
            let response = RoutingResponse {
                routed_to: route,
                token_count,
            };
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&response).unwrap()))
                .unwrap())
        }
        Err(e) => {
            eprintln!("❌ Routing error: {}", e);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Internal server error"}"#))
                .unwrap())
        }
    }
}

fn estimate_token_count(request: &RouterRequest) -> u32 {
    let mut chars = 0;
    
    // Messages
    for msg in &request.messages {
        match &msg.content {
            Value::String(s) => chars += s.len(),
            Value::Array(arr) => {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        chars += s.len();
                    }
                }
            }
            _ => {}
        }
    }
    
    // System prompt
    if let Some(system) = &request.system {
        chars += system.to_string().len();
    }
    
    // Tools
    if let Some(tools) = &request.tools {
        for tool in tools {
            chars += tool.name.len();
            if let Some(desc) = &tool.description {
                chars += desc.len();
            }
        }
    }
    
    (chars / 4) as u32
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