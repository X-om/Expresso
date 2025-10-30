use expresso::app::expresso::Expresso;
use expresso::http::request::Request;
use expresso::http::response::Response;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let app = Expresso::new();

    app.get(
        "/hello",
        (|_req: Request, res: Response, _next| async move {
            return res.status(200).send("Hello, World!");
        },),
    )
    .await;

    app.post(
        "/submit",
        (|req: Request, res: Response, _next| async move {
            let body: &String = match req.body() {
                Some(body) => body,
                None => {
                    return res.status(400).send("Bad Request");
                }
            };
            return res.json(&format!("{}", body));
        },),
    )
    .await;

    let port = 3000;
    app.listen(port, move || {
        println!("🚀 Server running at http://127.0.0.1:{}", port);
    })
    .await
}
