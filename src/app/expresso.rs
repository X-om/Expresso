/// Main Expresso application struct - simplified and modular
use crate::{
    handler::{IntoHandler, IntoHandlers},
    http::{request::Request, response::Response},
    middleware::MiddlewareManager,
    router::{Method, Router},
    server::listener::Server,
};
use std::{net::SocketAddr, sync::Arc};

/// The main Expresso application
///
/// This is your web framework entry point. Create an instance with `Expresso::new()`,
/// register routes and middleware, then call `listen()` to start the server.
pub struct Expresso {
    router: Arc<Router>,
    middleware: Arc<MiddlewareManager>,
}

impl Expresso {
    /// Create a new Expresso application instance
    ///
    /// # Example
    /// ```
    /// let app = Expresso::new();
    /// ```
    pub fn new() -> Self {
        Self {
            router: Arc::new(Router::new()),
            middleware: Arc::new(MiddlewareManager::new()),
        }
    }

    /// Register a global middleware
    ///
    /// Middleware runs for every request, in the order they are registered.
    ///
    /// # Example
    /// ```
    /// app.use_middleware(|req, res, next| async move {
    ///     println!("Request: {} {}", req.method(), req.path());
    ///     next().await
    /// }).await;
    /// ```
    pub async fn use_middleware<F>(&self, f: F)
    where
        F: IntoHandler,
    {
        self.middleware.add(f.into_handler()).await;
    }

    /// Register a GET route
    ///
    /// # Arguments
    /// * `path` - The URL path to match (e.g., "/users")
    /// * `handlers` - One or more handlers as a tuple
    ///
    /// # Example
    /// ```
    /// app.get("/hello", (|req, res, next| async move {
    ///     res.status(200).send("Hello!")
    /// },)).await;
    /// ```
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

    /// Start the HTTP server
    ///
    /// # Arguments
    /// * `port` - Port number to listen on
    /// * `callback` - Function called once when server starts
    ///
    /// # Example
    /// ```
    /// app.listen(3000, || {
    ///     println!("Server running on port 3000");
    /// }).await
    /// ```
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

                    // Execute the complete chain
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
