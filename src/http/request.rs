use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Request {
    pub fn new() -> Self { Self { method: String::new(), path: String::new(), version: String::new(), headers: HashMap::new(), body: None } }

    /// Parse raw HTTP request bytes into Request struct
    pub fn from_raw(buffer: &[u8]) -> Option<Self> {
        let request_str = String::from_utf8_lossy(buffer);
        let mut lines = request_str.split("\r\n");

        let request_line = lines.next()?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_string();
        let path = parts.next()?.to_string();
        let version = parts.next()?.to_string();
        let mut headers = HashMap::new();

        for line in &mut lines {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(": ") {
                headers.insert(key.to_string(), value.to_string());
            }
        }

        let body = lines.collect::<Vec<&str>>().join("\r\n");
        let body = if body.is_empty() { None } else { Some(body) };
        Some(Self { method, path, version, headers, body })
    }

    pub fn method(&self) -> &str { &self.method }

    pub fn path(&self) -> &str { &self.path }

    pub fn header(&self, key: &str) -> Option<&String> { self.headers.get(key) }

    pub fn body(&self) -> Option<&String> { self.body.as_ref() }
}
