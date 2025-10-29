use crate::http::request::Request;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

pub struct Server {
    addr: SocketAddr,
}

impl Server {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn listen<H, F>(&self, handler: H) -> tokio::io::Result<()>
    where
        H: Fn(Request) -> F + Send + Sync + 'static + Clone,
        F: std::future::Future<Output = crate::http::response::Response> + Send + 'static,
    {
        let listener: TcpListener = TcpListener::bind(self.addr).await?;
        loop {
            let (mut stream, _addr) = listener.accept().await?;
            let handler = handler.clone();

            tokio::spawn(async move {
                let mut buffer = vec![0; 4096];
                let n = stream.read(&mut buffer).await.ok()?;
                if n == 0 {
                    return Some(());
                }

                if let Some(req) = crate::http::request::Request::from_raw(&buffer[..n]) {
                    let res: crate::http::response::Response = handler(req).await;
                    let _ = stream.write_all(res.build().as_bytes()).await;
                }

                Some(())
            });
        }
    }
}
