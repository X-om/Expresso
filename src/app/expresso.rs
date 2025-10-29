use crate::{
    http::{request::Request, response::Response},
    server::listener::Server,
};
use std::{collections::HashMap, future::Future, net::SocketAddr, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Next {
    pub called: Arc<RwLock<bool>>,
}

impl Next {
    pub fn new() -> Self {
        Self {
            called: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn call(&self) {
        let mut called = self.called.write().await;
        *called = true;
    }

    pub async fn is_called(&self) -> bool {
        *self.called.read().await
    }
}

// Each handler: async closure that takes req, res, next and returns Response
pub type Handler = Arc<
    dyn Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync,
>;

pub struct Expresso {
    routes: Arc<RwLock<HashMap<String, Handler>>>,
}

impl Expresso {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get<F, Fut>(&self, path: &str, handler: F)
    where
        F: Fn(Request, Response, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let mut routes = futures::executor::block_on(self.routes.write());
        routes.insert(
            format!("GET:{}", path),
            Arc::new(move |req, res, next| Box::pin(handler(req, res, next))),
        );
    }

    pub fn post<F, Fut>(&self, path: &str, handler: F)
    where
        F: Fn(Request, Response, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let mut routes = futures::executor::block_on(self.routes.write());
        routes.insert(
            format!("POST:{}", path),
            Arc::new(move |req, res, next| Box::pin(handler(req, res, next))),
        );
    }

    pub async fn listen<F>(&self, port: u16, callback: F) -> tokio::io::Result<()>
    where
        F: Fn(),
    {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let server = Server::new(addr);
        let routes = Arc::clone(&self.routes);
        callback();

        server
            .listen(move |req: Request| {
                let routes = routes.clone();
                async move {
                    let key = format!("{}:{}", req.method(), req.path());
                    let routes = routes.read().await;

                    if let Some(handler) = routes.get(&key) {
                        let res = Response::new();
                        let next = Next::new();
                        handler(req, res, next).await
                    } else {
                        Response::new().status(404).send("Not Found")
                    }
                }
            })
            .await
    }
}
