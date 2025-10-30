use expresso::app::expresso::Expresso;
use expresso::app::expresso::Next;
use expresso::http::request::Request;
use expresso::http::response::Response;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let app = Expresso::new();

    app.get(
        "/hello",
        |_req: Request, res: Response, _next: Next| async move {
            return res.status(200).send("Hello, World!");
        },
    );

    app.post(
        "/submit",
        |req: Request, res: Response, _next: Next| async move {
            let body: &String = match req.body() {
                Some(body) => body,
                None => {
                    return res.status(400).send("Bad Request");
                }
            };
            return res.json(&format!("{}", body));
        },
    );

    let port = 3000;
    app.listen(port, || {
        println!("Server is running on http://localhost:{}", port);
    })
    .await
}
