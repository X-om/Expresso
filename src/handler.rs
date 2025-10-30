/// Handler traits for converting functions into the Handler type
use crate::{
    http::{request::Request, response::Response},
    types::{BoxFuture, Handler, Next},
};
use std::{future::Future, sync::Arc};

/// Trait for converting a function into a Handler
pub trait IntoHandler {
    fn into_handler(self) -> Handler;
}

/// Implement IntoHandler for any function that matches the signature
impl<F, Fut> IntoHandler for F
where
    F: Fn(Request, Response, Next) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    fn into_handler(self) -> Handler {
        Arc::new(move |req, res, next| Box::pin(self(req, res, next)))
    }
}

/// Trait for converting tuples of handlers into a single chained handler
pub trait IntoHandlers: Sized {
    fn into_handler_vec(self) -> Vec<Handler>;

    fn into_chained_handler(self) -> Handler {
        let handlers = self.into_handler_vec();
        if handlers.is_empty() {
            return Arc::new(|_req, res, _next| Box::pin(async move { res }));
        }

        Arc::new(move |req, res, final_next| {
            let handlers = handlers.clone();
            Box::pin(async move { execute_handlers(req, res, handlers, 0, final_next).await })
        })
    }
}

/// Recursively execute a chain of handlers
fn execute_handlers(
    req: Request,
    res: Response,
    handlers: Vec<Handler>,
    index: usize,
    final_next: Next,
) -> BoxFuture {
    Box::pin(async move {
        if index >= handlers.len() {
            return res;
        }

        let handler = handlers[index].clone();
        let req_clone = req.clone();
        let res_clone = res.clone();
        let handlers_clone = handlers.clone();

        handler(
            req,
            res,
            Arc::new(move || {
                let req_clone = req_clone.clone();
                let res_clone = res_clone.clone();
                let handlers_clone = handlers_clone.clone();
                let final_next = final_next.clone();
                Box::pin(async move {
                    execute_handlers(req_clone, res_clone, handlers_clone, index + 1, final_next)
                        .await
                })
            }),
        )
        .await
    })
}

// Implement IntoHandlers for tuples of different sizes

impl<A> IntoHandlers for (A,)
where
    A: IntoHandler,
{
    fn into_handler_vec(self) -> Vec<Handler> {
        vec![self.0.into_handler()]
    }
}

impl<A, B> IntoHandlers for (A, B)
where
    A: IntoHandler,
    B: IntoHandler,
{
    fn into_handler_vec(self) -> Vec<Handler> {
        vec![self.0.into_handler(), self.1.into_handler()]
    }
}

impl<A, B, C> IntoHandlers for (A, B, C)
where
    A: IntoHandler,
    B: IntoHandler,
    C: IntoHandler,
{
    fn into_handler_vec(self) -> Vec<Handler> {
        vec![
            self.0.into_handler(),
            self.1.into_handler(),
            self.2.into_handler(),
        ]
    }
}

impl<A, B, C, D> IntoHandlers for (A, B, C, D)
where
    A: IntoHandler,
    B: IntoHandler,
    C: IntoHandler,
    D: IntoHandler,
{
    fn into_handler_vec(self) -> Vec<Handler> {
        vec![
            self.0.into_handler(),
            self.1.into_handler(),
            self.2.into_handler(),
            self.3.into_handler(),
        ]
    }
}

impl<A, B, C, D, E> IntoHandlers for (A, B, C, D, E)
where
    A: IntoHandler,
    B: IntoHandler,
    C: IntoHandler,
    D: IntoHandler,
    E: IntoHandler,
{
    fn into_handler_vec(self) -> Vec<Handler> {
        vec![
            self.0.into_handler(),
            self.1.into_handler(),
            self.2.into_handler(),
            self.3.into_handler(),
            self.4.into_handler(),
        ]
    }
}

impl<A, B, C, D, E, F> IntoHandlers for (A, B, C, D, E, F)
where
    A: IntoHandler,
    B: IntoHandler,
    C: IntoHandler,
    D: IntoHandler,
    E: IntoHandler,
    F: IntoHandler,
{
    fn into_handler_vec(self) -> Vec<Handler> {
        vec![
            self.0.into_handler(),
            self.1.into_handler(),
            self.2.into_handler(),
            self.3.into_handler(),
            self.4.into_handler(),
            self.5.into_handler(),
        ]
    }
}

impl<A, B, C, D, E, F, G> IntoHandlers for (A, B, C, D, E, F, G)
where
    A: IntoHandler,
    B: IntoHandler,
    C: IntoHandler,
    D: IntoHandler,
    E: IntoHandler,
    F: IntoHandler,
    G: IntoHandler,
{
    fn into_handler_vec(self) -> Vec<Handler> {
        vec![
            self.0.into_handler(),
            self.1.into_handler(),
            self.2.into_handler(),
            self.3.into_handler(),
            self.4.into_handler(),
            self.5.into_handler(),
            self.6.into_handler(),
        ]
    }
}

impl<A, B, C, D, E, F, G, H> IntoHandlers for (A, B, C, D, E, F, G, H)
where
    A: IntoHandler,
    B: IntoHandler,
    C: IntoHandler,
    D: IntoHandler,
    E: IntoHandler,
    F: IntoHandler,
    G: IntoHandler,
    H: IntoHandler,
{
    fn into_handler_vec(self) -> Vec<Handler> {
        vec![
            self.0.into_handler(),
            self.1.into_handler(),
            self.2.into_handler(),
            self.3.into_handler(),
            self.4.into_handler(),
            self.5.into_handler(),
            self.6.into_handler(),
            self.7.into_handler(),
        ]
    }
}
