use expresso::{app::expresso::Expresso, http::response::Response};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let app = Expresso::new();

    app.get("/hello", |_req| Response::new().send("Hello GET")).await;
    app.post("/submit", |_req| Response::new().json("{\"ok\": true}")).await;

    app.listen("127.0.0.1:3000").await
}
