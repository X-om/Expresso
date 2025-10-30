use crate::http::{request::Request, response::Response};
use std::{future::Future, pin::Pin, sync::Arc};

pub type Next = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;
pub type Handler = Arc<
    dyn Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync,
>;
pub type BoxFuture = Pin<Box<dyn Future<Output = Response> + Send>>;
