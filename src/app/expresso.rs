use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    http::{request::Request, response::Response},
    server::listener::Server,
};

type Handler = Arc<dyn Fn(Request) -> Response + Send + Sync>;

pub struct Expresso {
    routes: Arc<RwLock<HashMap<String, Handler>>>,
}

impl Expresso {
    pub fn new() -> Self { Self { routes: Arc::new(RwLock::new(HashMap::new())) } }

    pub async fn get<F>(&self, path: &str, handler: F)
    where F: Fn(Request) -> Response + Send + Sync + 'static {
        let mut routes = self.routes.write().await;
        routes.insert(format!("GET:{}", path), Arc::new(handler));
    }

    pub async fn post<F>(&self, path: &str, handler: F)
    where F: Fn(Request) -> Response + Send + Sync + 'static {
        let mut routes = self.routes.write().await;
        routes.insert(format!("POST:{}", path), Arc::new(handler));
    }

    pub async fn listen(&self, addr: &str) -> tokio::io::Result<()> {
        let socket: SocketAddr = addr.parse().unwrap();
        let server = Server::new(socket);
        let routes = Arc::clone(&self.routes);

        server
            .listen_with_handler(move |req: Request| {
                let routes = routes.clone();
                async move {
                    let key = format!("{}:{}", req.method(), req.path());
                    let routes = routes.read().await;

                    if let Some(handler) = routes.get(&key) {
                        handler(req)
                    } else {
                        Response::new().status(404).send("Not Found")
                    }
                }
            })
            .await
    }
}
