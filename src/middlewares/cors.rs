use crate::{
    http::{request::Request, response::Response},
    types::Next,
};

pub async fn cors(_req: Request, _res: Response, next: Next) -> Response {
    let res = next().await;
    res.set_header("Access-Control-Allow-Origin", "*")
        .set_header(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, PATCH, OPTIONS",
        )
        .set_header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )
}

pub fn with_origin(
    origin: &'static str,
) -> impl Fn(
    Request,
    Response,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Send
       + Sync
       + 'static {
    move |_req: Request, _res: Response, next: Next| {
        Box::pin(async move {
            let res = next().await;
            res.set_header("Access-Control-Allow-Origin", origin)
                .set_header(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, DELETE, PATCH, OPTIONS",
                )
                .set_header(
                    "Access-Control-Allow-Headers",
                    "Content-Type, Authorization",
                )
        })
    }
}

/// Custom CORS configuration
pub struct CorsConfig {
    pub origins: Vec<String>,
    pub methods: Vec<String>,
    pub headers: Vec<String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            origins: vec!["*".to_string()],
            methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
        }
    }
}

impl CorsConfig {
    /// Create a new CORS configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set allowed origins
    pub fn origins(mut self, origins: Vec<String>) -> Self {
        self.origins = origins;
        self
    }

    /// Set allowed methods
    pub fn methods(mut self, methods: Vec<String>) -> Self {
        self.methods = methods;
        self
    }

    /// Set allowed headers
    pub fn headers(mut self, headers: Vec<String>) -> Self {
        self.headers = headers;
        self
    }

    /// Build the middleware function
    pub fn build(
        self,
    ) -> impl Fn(
        Request,
        Response,
        Next,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
           + Send
           + Sync
           + 'static {
        let origins = self.origins.join(", ");
        let methods = self.methods.join(", ");
        let headers = self.headers.join(", ");

        move |_req: Request, _res: Response, next: Next| {
            let origins = origins.clone();
            let methods = methods.clone();
            let headers = headers.clone();

            Box::pin(async move {
                let res = next().await;
                res.set_header("Access-Control-Allow-Origin", &origins)
                    .set_header("Access-Control-Allow-Methods", &methods)
                    .set_header("Access-Control-Allow-Headers", &headers)
            })
        }
    }
}
