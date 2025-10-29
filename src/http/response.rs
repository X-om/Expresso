use std::collections::HashMap;

pub struct Response {
    status_code: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl Response {
    pub fn new() -> Self { return Self { status_code: 200, status_text: "OK".to_string(), headers: HashMap::new(), body: None }; }

    pub fn status(mut self, code: u16) -> Self {
        self.status_code = code;
        let code = match code {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };
        self.status_text = String::from(code);
        return self;
    }

    pub fn set_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        return self;
    }

    pub fn send(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        return self;
    }

    pub fn json(mut self, data: &str) -> Self {
        self.headers.insert("Content-Type".into(), "application/json".into());
        self.body = Some(data.to_string());
        return self;
    }

    pub fn build(&self) -> String {
        let body_str = self.body.clone().unwrap_or_default();
        let content_length = body_str.len();
        let mut headers = String::new();

        for (k, v) in &self.headers {
            headers.push_str(&format!("{}: {}\r\n", k, v));
        }
        return format!("HTTP/1.1 {} {}\r\nContent-Length: {}\r\n{}\r\n{}", self.status_code, self.status_text, content_length, headers, body_str);
    }
}
