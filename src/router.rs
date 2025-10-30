use crate::types::Handler;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl Method {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(Method::GET),
            "POST" => Some(Method::POST),
            "PUT" => Some(Method::PUT),
            "DELETE" => Some(Method::DELETE),
            "PATCH" => Some(Method::PATCH),
            "HEAD" => Some(Method::HEAD),
            "OPTIONS" => Some(Method::OPTIONS),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::PATCH => "PATCH",
            Method::HEAD => "HEAD",
            Method::OPTIONS => "OPTIONS",
        }
    }
}

pub struct Router {
    routes: RwLock<HashMap<String, Handler>>,
}

impl Router {
    /// Create a new empty router
    pub fn new() -> Self {
        Self {
            routes: RwLock::new(HashMap::new()),
        }
    }

    /// Register a route with a handler
    pub async fn add_route(&self, method: Method, path: &str, handler: Handler) {
        let key = format!("{}:{}", method.as_str(), path);
        let mut routes = self.routes.write().await;
        routes.insert(key, handler);
    }

    /// Find a handler for the given method and path
    pub async fn find_handler(&self, method: &str, path: &str) -> Option<Handler> {
        let key = format!("{}:{}", method, path);
        let routes = self.routes.read().await;
        routes.get(&key).cloned()
    }

    /// Get all registered routes (useful for debugging)
    pub async fn get_all_routes(&self) -> Vec<String> {
        let routes = self.routes.read().await;
        routes.keys().cloned().collect()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
