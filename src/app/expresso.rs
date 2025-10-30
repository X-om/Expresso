/// Main Expresso application struct - simplified and modular
use crate::{
    handler::{IntoHandler, IntoHandlers},
    http::{request::Request, response::Response},
    middleware::MiddlewareManager,
    router::{Method, Router},
    server::listener::Server,
};
use std::{net::SocketAddr, sync::Arc};

pub struct Expresso {
    router: Arc<Router>,
    middleware: Arc<MiddlewareManager>,
}

impl Expresso {
    pub fn new() -> Self {
        Self {
            router: Arc::new(Router::new()),
            middleware: Arc::new(MiddlewareManager::new()),
        }
    }

    pub async fn use_middleware<F>(&self, f: F)
    where
        F: IntoHandler,
    {
        self.middleware.add(f.into_handler()).await;
    }

    pub async fn get<H>(&self, path: &str, handlers: H)
    where
        H: IntoHandlers,
    {
        self.router
            .add_route(Method::GET, path, handlers.into_chained_handler())
            .await;
    }

    /// Register a POST route
    pub async fn post<H>(&self, path: &str, handlers: H)
    where
        H: IntoHandlers,
    {
        self.router
            .add_route(Method::POST, path, handlers.into_chained_handler())
            .await;
    }

    /// Register a PUT route
    pub async fn put<H>(&self, path: &str, handlers: H)
    where
        H: IntoHandlers,
    {
        self.router
            .add_route(Method::PUT, path, handlers.into_chained_handler())
            .await;
    }

    /// Register a DELETE route
    pub async fn delete<H>(&self, path: &str, handlers: H)
    where
        H: IntoHandlers,
    {
        self.router
            .add_route(Method::DELETE, path, handlers.into_chained_handler())
            .await;
    }

    /// Register a PATCH route
    pub async fn patch<H>(&self, path: &str, handlers: H)
    where
        H: IntoHandlers,
    {
        self.router
            .add_route(Method::PATCH, path, handlers.into_chained_handler())
            .await;
    }

    pub async fn listen<F>(&self, port: u16, callback: F) -> tokio::io::Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        let server = Server::new(addr);
        let router = Arc::clone(&self.router);
        let middleware = Arc::clone(&self.middleware);

        callback();

        server
            .listen(move |req: Request| {
                let router = router.clone();
                let middleware = middleware.clone();

                async move {
                    let res = Response::new();
                    let method = req.method();
                    let path = req.path();

                    // Find the route handler or use 404 handler
                    let route_handler =
                        router.find_handler(method, path).await.unwrap_or_else(|| {
                            Arc::new(|_req, res, _next| {
                                Box::pin(async move { res.status(404).send("Not Found") })
                            })
                        });

                    // Build middleware chain that wraps the route handler
                    let chain = middleware.build_chain(route_handler).await;
                    chain(
                        req,
                        res,
                        Arc::new(|| {
                            Box::pin(async { Response::new().status(500).send("Internal Error") })
                        }),
                    )
                    .await
                }
            })
            .await
    }

    /// Get all registered routes (useful for debugging)
    pub async fn routes(&self) -> Vec<String> {
        self.router.get_all_routes().await
    }
}

impl Default for Expresso {
    fn default() -> Self {
        Self::new()
    }
}
