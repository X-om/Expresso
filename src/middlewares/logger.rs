/// Logger middleware - logs incoming requests
use crate::{
    http::{request::Request, response::Response},
    types::Next,
};

/// Simple logger that prints request method and path
///
/// # Example
/// ```
/// use expresso::middlewares::logger::logger;
///
/// app.use_middleware(logger).await;
/// ```
pub async fn logger(req: Request, _res: Response, next: Next) -> Response {
    println!("📝 {} {}", req.method(), req.path());
    next().await
}

/// Detailed logger that includes headers
///
/// # Example
/// ```
/// use expresso::middlewares::logger::detailed_logger;
///
/// app.use_middleware(detailed_logger).await;
/// ```
pub async fn detailed_logger(req: Request, _res: Response, next: Next) -> Response {
    println!(
        "📝 [Logger] {} {} - Headers: {:?}",
        req.method(),
        req.path(),
        req.headers
    );
    next().await
}

/// Logger with custom prefix
///
/// # Example
/// ```
/// use expresso::middlewares::logger::with_prefix;
///
/// app.use_middleware(with_prefix("[API]")).await;
/// ```
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
