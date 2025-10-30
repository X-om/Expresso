use expresso::middlewares::{cors, logger};
use expresso::prelude::*;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let app = Expresso::new();

    // Use built-in middleware
    app.use_middleware(logger::detailed_logger).await;
    app.use_middleware(cors::cors).await;

    // Custom middleware
    app.use_middleware(|req: Request, _res: Response, next: Next| async move {
        if let Some(auth_header) = req.header("Authorization") {
            println!("🔐 [Auth] Authorized: {}", auth_header);
        } else {
            println!("⚠️  [Auth] No authorization header");
        }
        next().await
    })
    .await;

    // Simple route
    app.get(
        "/hello",
        (|_req: Request, res: Response, _next: Next| async move {
            res.status(200).send("Hello, World! 👋")
        },),
    )
    .await;

    // Protected route with middleware chain
    app.get(
        "/protected",
        (
            |req: Request, res: Response, next: Next| async move {
                if let Some(role) = req.header("X-User-Role") {
                    if role == "admin" {
                        println!("✅ Admin access granted");
                        next().await
                    } else {
                        res.status(403).send("Forbidden: Admin role required")
                    }
                } else {
                    res.status(403).send("Forbidden: No role header")
                }
            },
            |_req: Request, res: Response, _next: Next| async move {
                res.status(200).send("🎉 Welcome to the protected area!")
            },
        ),
    )
    .await;

    // POST route with body
    app.post(
        "/submit",
        (|req: Request, res: Response, _next: Next| async move {
            match req.body() {
                Some(body) => {
                    println!("📨 Received: {}", body);
                    res.status(201)
                        .set_header("Content-Type", "application/json")
                        .json(&format!(r#"{{"message":"Data received","data":{}}}"#, body))
                }
                None => res.status(400).send("Bad Request: No body"),
            }
        },),
    )
    .await;

    // Multiple HTTP methods
    app.put(
        "/update",
        (|_req: Request, res: Response, _next: Next| async move {
            res.status(200).send("Resource updated")
        },),
    )
    .await;

    app.delete(
        "/delete",
        (|_req: Request, res: Response, _next: Next| async move {
            res.status(200).send("Resource deleted")
        },),
    )
    .await;

    let port = 3000;
    app.listen(port, move || {
        println!("\n╔════════════════════════════════════════╗");
        println!("║  🚀 Expresso Server Started!          ║");
        println!("╠════════════════════════════════════════╣");
        println!("║  📍 Address: http://127.0.0.1:{}     ║", port);
        println!("╠════════════════════════════════════════╣");
        println!("║  Available Routes:                     ║");
        println!("║  • GET    /hello                       ║");
        println!("║  • GET    /protected                   ║");
        println!("║  • POST   /submit                      ║");
        println!("║  • PUT    /update                      ║");
        println!("║  • DELETE /delete                      ║");
        println!("╠════════════════════════════════════════╣");
        println!("║  💡 Test with curl:                    ║");
        println!("║  curl http://127.0.0.1:{}       ║", port);
        println!("╚════════════════════════════════════════╝\n");
    })
    .await
}
