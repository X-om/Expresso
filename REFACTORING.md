# Expresso Framework - Refactored Structure

A modular, Express.js-inspired HTTP framework for Rust, built with Tokio.

## 📁 Project Structure

```
src/
├── lib.rs              # Library entry point with prelude
├── main.rs             # Example application
├── types.rs            # Common type definitions (Handler, Next, etc.)
├── router.rs           # Route management and HTTP methods
├── middleware.rs       # Middleware chain execution
├── handler.rs          # Handler traits and tuple implementations
├── app/
│   └── expresso.rs     # Main Expresso application struct
├── http/
│   ├── request.rs      # HTTP Request struct
│   └── response.rs     # HTTP Response struct
├── server/
│   ├── listener.rs     # TCP server and connection handling
│   └── parser.rs       # HTTP parsing utilities
├── middlewares/
│   ├── logger.rs       # Built-in logging middleware
│   └── cors.rs         # Built-in CORS middleware
└── errors/
    └── mod.rs          # Error types (to be implemented)
```

## 🎯 Module Responsibilities

### **types.rs**
- Defines common type aliases used throughout the framework
- `Handler`: Function signature for request handlers
- `Next`: Function to call next middleware/handler
- `BoxFuture`: Pinned async future returning Response

### **router.rs**
- `Router`: Manages route registration and matching
- `Method`: Enum for HTTP methods (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
- Methods: `add_route()`, `find_handler()`, `get_all_routes()`

### **middleware.rs**
- `MiddlewareManager`: Manages global middleware stack
- Methods: `add()`, `build_chain()`, `count()`
- Handles middleware execution order and chaining

### **handler.rs**
- `IntoHandler`: Trait for converting functions to handlers
- `IntoHandlers`: Trait for converting tuples to handler chains
- Implementations for tuples of 1-8 handlers
- `execute_handlers()`: Recursive handler execution

### **app/expresso.rs**
- `Expresso`: Main application struct
- Public API methods:
  - `new()`: Create new app instance
  - `use_middleware()`: Register global middleware
  - `get()`, `post()`, `put()`, `delete()`, `patch()`: Register routes
  - `listen()`: Start the server
  - `routes()`: Get all registered routes

### **middlewares/**
Built-in middleware for common use cases:

#### logger.rs
- `logger()`: Simple request logging
- `detailed_logger()`: Logs with headers
- `with_prefix()`: Custom prefix logger

#### cors.rs
- `cors()`: Default CORS (allow all)
- `with_origin()`: CORS with specific origin
- `CorsConfig`: Builder for custom CORS configuration

## 🚀 Quick Start

### Basic Example

```rust
use expresso::prelude::*;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let app = Expresso::new();

    // Register middleware
    app.use_middleware(|req, res, next| async move {
        println!("Request: {}", req.path());
        next().await
    }).await;

    // Register route
    app.get("/hello", (|_req, res, _next| async move {
        res.status(200).send("Hello, World!")
    },)).await;

    // Start server
    app.listen(3000, || {
        println!("Server running on port 3000");
    }).await
}
```

### Using Built-in Middleware

```rust
use expresso::prelude::*;
use expresso::middlewares::{logger, cors};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let app = Expresso::new();

    // Use built-in middleware
    app.use_middleware(logger::detailed_logger).await;
    app.use_middleware(cors::cors).await;

    app.get("/api", (|_req, res, _next| async move {
        res.status(200).json(r#"{"status":"ok"}"#)
    },)).await;

    app.listen(3000, || println!("Started!")).await
}
```

### Route with Multiple Handlers

```rust
app.get("/protected", (
    // Middleware 1: Auth check
    |req, res, next| async move {
        if req.header("Authorization").is_some() {
            next().await
        } else {
            res.status(401).send("Unauthorized")
        }
    },
    // Middleware 2: Log access
    |req, res, next| async move {
        println!("Accessing protected route");
        next().await
    },
    // Final handler
    |_req, res, _next| async move {
        res.status(200).send("Protected data")
    },
)).await;
```

### Multiple HTTP Methods

```rust
app.get("/resource", (|_req, res, _next| async move {
    res.status(200).send("GET")
},)).await;

app.post("/resource", (|_req, res, _next| async move {
    res.status(201).send("Created")
},)).await;

app.put("/resource", (|_req, res, _next| async move {
    res.status(200).send("Updated")
},)).await;

app.delete("/resource", (|_req, res, _next| async move {
    res.status(200).send("Deleted")
},)).await;
```

## 🔧 Adding New Features

### Creating Custom Middleware

1. **Simple function**:
```rust
async fn my_middleware(req: Request, res: Response, next: Next) -> Response {
    // Do something before
    let response = next().await;
    // Do something after
    response
}

app.use_middleware(my_middleware).await;
```

2. **With configuration**:
```rust
fn with_config(config: &'static str) -> impl Fn(Request, Response, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync + 'static {
    move |req, res, next| {
        Box::pin(async move {
            println!("Config: {}", config);
            next().await
        })
    }
}

app.use_middleware(with_config("production")).await;
```

### Adding New HTTP Methods

In `router.rs`, add to the `Method` enum:
```rust
pub enum Method {
    // ... existing methods
    CONNECT,
}
```

In `expresso.rs`, add the method:
```rust
pub async fn connect<H>(&self, path: &str, handlers: H)
where
    H: IntoHandlers,
{
    self.router
        .add_route(Method::CONNECT, path, handlers.into_chained_handler())
        .await;
}
```

### Creating New Middleware Modules

1. Create file in `src/middlewares/my_middleware.rs`
2. Add to `src/middlewares/mod.rs`: `pub mod my_middleware;`
3. Implement your middleware functions

## 📦 Prelude

The prelude exports commonly used items:
```rust
use expresso::prelude::*;
// Imports: Expresso, Request, Response, Handler, Next, Method, IntoHandler, IntoHandlers
```

## 🎓 Key Rust Concepts Used

1. **Traits**: `IntoHandler`, `IntoHandlers` for conversion patterns
2. **Generics**: Flexible handler signatures with `<F>`, `<H>`
3. **Async/Await**: Non-blocking I/O throughout
4. **Arc + RwLock**: Thread-safe shared state
5. **Closures**: Handler and middleware functions
6. **Pattern Matching**: Route matching and error handling
7. **Type Aliases**: Simplifying complex types
8. **Builder Pattern**: `CorsConfig` for configuration

## 🔄 Migration from Old Code

Old:
```rust
use expresso::app::expresso::Expresso;
use expresso::http::request::Request;
use expresso::http::response::Response;
use std::{future::Future, pin::Pin, sync::Arc};
type Next = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;
```

New:
```rust
use expresso::prelude::*;
// That's it! Next, Request, Response are all imported
```

## 📈 Benefits of Refactoring

1. **Modularity**: Each module has a single, clear responsibility
2. **Extensibility**: Easy to add new middleware, methods, or features
3. **Reusability**: Built-in middleware can be imported and used
4. **Clarity**: Separation of concerns makes code easier to understand
5. **Maintainability**: Changes to one module don't affect others
6. **Testing**: Each module can be tested independently
7. **Documentation**: Clear structure makes it easier to document

## 🚧 Future Enhancements

- [ ] Error handling module with custom error types
- [ ] Body parsing middleware (JSON, form data)
- [ ] Static file serving
- [ ] Session management
- [ ] Authentication helpers
- [ ] Rate limiting middleware
- [ ] Compression middleware
- [ ] Routing parameters (e.g., `/users/:id`)
- [ ] Query string parsing
- [ ] WebSocket support

## 📝 License

MIT
