use crate::{
    http::{request::Request, response::Response},
    server::listener::Server,
};
use std::{collections::HashMap, future::Future, net::SocketAddr, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

type Next = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

type Handler = Arc<
    dyn Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync,
>;

pub struct Expresso {
    routes: Arc<RwLock<HashMap<String, Handler>>>,
    middlewares: Arc<RwLock<Vec<Handler>>>,
}

impl Expresso {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
            middlewares: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn use_middleware<F>(&self, f: F)
    where
        F: IntoHandler,
    {
        let mut middlewares = self.middlewares.write().await;
        middlewares.push(f.into_handler());
    }

    pub async fn get<H>(&self, path: &str, handlers: H)
    where
        H: IntoHandlers,
    {
        let mut routes = self.routes.write().await;
        routes.insert(format!("GET:{}", path), handlers.into_chained_handler());
    }

    pub async fn post<H>(&self, path: &str, handlers: H)
    where
        H: IntoHandlers,
    {
        let mut routes = self.routes.write().await;
        routes.insert(format!("POST:{}", path), handlers.into_chained_handler());
    }

    pub async fn listen<F>(&self, port: u16, callback: F) -> tokio::io::Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        let server = Server::new(addr);
        let routes = Arc::clone(&self.routes);
        let middlewares = Arc::clone(&self.middlewares);

        callback();

        server
            .listen(move |req: Request| {
                let routes = routes.clone();
                let middlewares = middlewares.clone();

                async move {
                    let res = Response::new();
                    let key = format!("{}:{}", req.method(), req.path());

                    let route_handler = {
                        let routes = routes.read().await;
                        routes.get(&key).cloned()
                    };

                    let final_handler: Handler = if let Some(h) = route_handler {
                        h
                    } else {
                        Arc::new(|_req, res, _next| {
                            Box::pin(async move { res.status(404).send("Not Found") })
                        })
                    };

                    let middlewares = middlewares.read().await.clone();
                    let chain =
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
                                                                Response::new()
                                                                    .status(500)
                                                                    .send("Internal Error")
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
                            });

                    // Execute chain
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
}

pub trait IntoHandler {
    fn into_handler(self) -> Handler;
}

impl<F, Fut> IntoHandler for F
where
    F: Fn(Request, Response, Next) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    fn into_handler(self) -> Handler {
        Arc::new(move |req, res, next| Box::pin(self(req, res, next)))
    }
}

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

// Helper function to recursively execute handlers
fn execute_handlers(
    req: Request,
    res: Response,
    handlers: Vec<Handler>,
    index: usize,
    final_next: Next,
) -> Pin<Box<dyn Future<Output = Response> + Send>> {
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

// Single handler
impl<A> IntoHandlers for (A,)
where
    A: IntoHandler,
{
    fn into_handler_vec(self) -> Vec<Handler> {
        vec![self.0.into_handler()]
    }
}

// Two handlers
impl<A, B> IntoHandlers for (A, B)
where
    A: IntoHandler,
    B: IntoHandler,
{
    fn into_handler_vec(self) -> Vec<Handler> {
        vec![self.0.into_handler(), self.1.into_handler()]
    }
}

// Three handlers
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

// Four handlers
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

// Five handlers
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

// Six handlers
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

// Seven handlers
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

// Eight handlers
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
