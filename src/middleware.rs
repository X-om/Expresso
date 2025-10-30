/// Middleware management and execution
use crate::{
    http::{request::Request, response::Response},
    types::{Handler, Next},
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Middleware manager stores and executes middleware chain
pub struct MiddlewareManager {
    middlewares: RwLock<Vec<Handler>>,
}

impl MiddlewareManager {
    /// Create a new middleware manager
    pub fn new() -> Self {
        Self {
            middlewares: RwLock::new(Vec::new()),
        }
    }

    /// Add a middleware to the stack
    pub async fn add(&self, middleware: Handler) {
        let mut middlewares = self.middlewares.write().await;
        middlewares.push(middleware);
    }

    /// Build a middleware chain that wraps the final handler
    /// Middlewares execute in the order they were added
    pub async fn build_chain(&self, final_handler: Handler) -> Handler {
        let middlewares = self.middlewares.read().await.clone();

        // Fold middlewares in reverse order to build the chain
        // Last middleware wraps the final handler, second-to-last wraps that, etc.
        middlewares
            .into_iter()
            .rev()
            .fold(final_handler, |next_handler, mw| {
                Arc::new(move |req: Request, res: Response, _final_next: Next| {
                    let next_handler = next_handler.clone();
                    let mw = mw.clone();

                    Box::pin(async move {
                        let req_clone = req.clone();
                        let res_clone = res.clone();

                        // Call the middleware with a "next" function
                        mw(
                            req,
                            res,
                            Arc::new(move || {
                                let next_handler = next_handler.clone();
                                let req_clone = req_clone.clone();
                                let res_clone = res_clone.clone();

                                Box::pin(async move {
                                    next_handler(
                                        req_clone,
                                        res_clone,
                                        Arc::new(|| {
                                            Box::pin(async {
                                                Response::new().status(500).send("Internal Error")
                                            })
                                        }),
                                    )
                                    .await
                                })
                            }),
                        )
                        .await
                    })
                })
            })
    }

    /// Get count of registered middlewares
    pub async fn count(&self) -> usize {
        let middlewares = self.middlewares.read().await;
        middlewares.len()
    }
}

impl Default for MiddlewareManager {
    fn default() -> Self {
        Self::new()
    }
}
