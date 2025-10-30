/// Logger middleware - logs incoming requests
use crate::{
    http::{request::Request, response::Response},
    types::Next,
};

pub async fn logger(req: Request, _res: Response, next: Next) -> Response {
    println!("📝 {} {}", req.method(), req.path());
    next().await
}

pub async fn detailed_logger(req: Request, _res: Response, next: Next) -> Response {
    println!(
        "📝 [Logger] {} {} - Headers: {:?}",
        req.method(),
        req.path(),
        req.headers
    );
    next().await
}

pub fn with_prefix(
    prefix: &'static str,
) -> impl Fn(
    Request,
    Response,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Send
       + Sync
       + 'static {
    move |req: Request, _res: Response, next: Next| {
        Box::pin(async move {
            println!("{} {} {}", prefix, req.method(), req.path());
            next().await
        })
    }
}
