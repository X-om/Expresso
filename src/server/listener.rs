use std::net::SocketAddr;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::http::{request::Request, response::Response};

pub struct Server {
    addr: SocketAddr,
}

impl Server {
    pub fn new(addr: SocketAddr) -> Self { Self { addr } }

    // * Start listening for incoming TCP connections
    pub async fn listen(&self) -> tokio::io::Result<()> {
        let listener: TcpListener = TcpListener::bind(self.addr).await?;
        println!("Server listening on {}", self.addr);

        loop {
            let (stream, addr) = listener.accept().await?;
            tokio::spawn(async move {
                if let Err(e) = Server::handle_connection(stream, addr).await {
                    eprintln!("Failed to handle connection from {}: {}", addr, e);
                }
            });
        }
    }

    // * A simple HTTP handler that responds with a fixed message
    pub async fn handle_connection(mut stream: TcpStream, addr: SocketAddr) -> tokio::io::Result<()> {
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            return Ok(());
        }

        if let Some(req) = Request::from_raw(&buffer[..n]) {
            println!("📥 Incoming Request [{}] {} from {}", req.method(), req.path(), addr);

            // ! TO BE REMOVED LATER
            for (k, v) in &req.headers {
                println!(" {}: {}", k, v);
            }

            // ! TO BE REMOVED LATER
            if let Some(body) = req.body() {
                println!("  Body: {}", body);
            }

            let res = Response::new().status(200).set_header("Content-Type", "text/plain").send(&format!("Received {} request for {}", req.method(), req.path()));
            stream.write_all(res.build().as_bytes()).await?;
        } else {
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            stream.write_all(response.as_bytes()).await?;
        }

        Ok(())
    }

    pub async fn listen_with_handler<H, F>(&self, handler: H) -> tokio::io::Result<()>
    where
        H: Fn(crate::http::request::Request) -> F + Send + Sync + 'static + Clone,
        F: std::future::Future<Output = crate::http::response::Response> + Send + 'static,
    {
        let listener = TcpListener::bind(self.addr).await?;
        println!("🚀 Server running at http://{}", self.addr);

        loop {
            let (mut stream, addr) = listener.accept().await?;
            let handler = handler.clone();

            tokio::spawn(async move {
                let mut buffer = vec![0; 4096];
                let n = stream.read(&mut buffer).await.ok()?;
                if n == 0 {
                    return Some(());
                }

                if let Some(req) = crate::http::request::Request::from_raw(&buffer[..n]) {
                    let res = handler(req).await;
                    let _ = stream.write_all(res.build().as_bytes()).await;
                }

                Some(())
            });
        }
    }
}
