use expresso::app::expresso::Expresso;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let app = Expresso::new();

    app.get("/hello", |_req, res, _next| async move {
        return res.status(200).send("Hello, World!");
    });

    app.post("/submit", |req, res, _next| async move {
        let body: &String = match req.body() {
            Some(body) => body,
            None => {
                return res.status(400).send("Bad Request");
            }
        };
        return res.json(&format!("{}", body));
    });

    let port = 3000;
    app.listen(port, || {
        println!("Server is running on http://localhost:{}", port);
    })
    .await
}
