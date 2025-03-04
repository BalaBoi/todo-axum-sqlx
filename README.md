# Axum Todo Web App - Educational Project

This project serves as an educational resource for learning how to build scalable and robust web applications using the Axum framework in Rust. It demonstrates modern web development practices with a focus on maintainability, security, and proper architecture.

## 🎯 Project Purpose

This application was built as a learning project to understand:
- How to structure a professional Axum web application
- How to implement common web application patterns in Rust
- Best practices for authentication, error handling, and database integration
- How to create a maintainable and scalable codebase

## 📚 Skills & Concepts Demonstrated

### Custom Middleware
- Authentication middleware that protects routes (`auth_middleware`)
- Session management with Tower Sessions
- Request ID generation and propagation for tracing
- Structured logging with the TraceLayer

### Custom Extractors
- Flash message extractor for communicating between requests
- Form data extraction with validation
- Session data extraction and enhancement

### Authentication System
- Custom user authentication with session-based login
- Password hashing with Argon2 (industry standard)
- HMAC signing for secure data
- Protected routes with middleware guards

### Error Handling
- Centralized error type with conversions (`Error` enum)
- Constraint-based database error mapping (`ResultExt` trait)
- User-friendly error messages and redirects
- Consistent error responses across the application

### Tracing & Observability
- Request tracing with unique request IDs
- Structured logging with tracing-subscriber
- Span-based context propagation
- Environment-based log filtering

### Database Integration
- SQLx for type-safe database queries
- Migration management
- Connection pooling
- Repository pattern for database operations

### Template Rendering
- Server-side rendering with Askama templates
- Typed template contexts
- Reusable template components

### Testing
- Integration testing of API endpoints
- Test helpers for common operations
- Isolated test environment

### Configuration Management
- Environment-specific configuration
- YAML-based configuration
- Strongly-typed settings with validation

## 🏗️ Project Structure

```
├── config/               # Configuration files
│   ├── base.yaml         # Base configuration
│   └── dev.yaml          # Development environment config
├── migrations/           # Database migrations
├── scripts/              # Utility scripts
├── src/
│   ├── config.rs         # Configuration loading
│   ├── http/             # HTTP layer
│   │   ├── error.rs      # Error handling
│   │   ├── tasks/        # Task-related endpoints
│   │   ├── users/        # User-related endpoints
│   │   └── utilities.rs  # Common HTTP utilities
│   ├── lib.rs            # Library entry point
│   ├── logging.rs        # Logging setup
│   └── main.rs           # Application entry point
├── templates/            # HTML templates
└── tests/                # Integration tests
```

## 🚀 Getting Started

### Prerequisites

- Rust (stable) - [Install Rust](https://www.rust-lang.org/tools/install)
- PostgreSQL - [Install PostgreSQL](https://www.postgresql.org/download/)
- Docker (optional, for containerized database) - [Install Docker](https://docs.docker.com/get-docker/)

### Setting Up the Development Environment

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd todo-axum-sqlx
   ```

2. Create a `.env` file in the project root with the following configuration:
   ```
   DATABASE_URL=postgres://postgres:password@localhost:5432/todo
   POSTGRES_USER=postgres
   POSTGRES_PASSWORD=password
   POSTGRES_DB=todo
   POSTGRES_HOST=localhost
   POSTGRES_PORT=5432
   POSTGRES_CONTAINER_NAME=todo_db
   POSTGRES_IMAGE=postgres:14
   APP_ENV=dev
   ```

3. Start PostgreSQL database:
   
   **Option 1**: Using the provided script (requires Docker):
   ```bash
   chmod +x scripts/setup.sh
   ./scripts/setup.sh start
   ```
   
   **Option 2**: Using your locally installed PostgreSQL:
   ```bash
   createdb todo
   ```

4. Run database migrations:
   ```bash
   cargo install sqlx-cli --no-default-features --features native-tls,postgres
   sqlx db setup
   ```

5. Build and run the application:
   ```bash
   cargo run
   ```

6. Access the application:
   The application will be available at http://localhost:8000

### Running Tests

```bash
cargo test
```

## 📝 Key Features to Learn From

### Custom Middleware Implementation

The project includes a custom authentication middleware (`auth_middleware`) that protects routes by checking for valid user sessions:

```rust
pub async fn auth_middleware(session: Session, mut req: Request, next: Next) -> Result<Response> {
    match session.get::<UserSessionData>(UserSessionData::SESSION_KEY).await? {
        Some(user_session_data) => {
            req.extensions_mut().insert(user_session_data);
            let response = next.run(req).await;
            Ok(response)
        }
        None => Err(Error::Unauthorized),
    }
}
```

### Custom Error Handling

The project implements a central error type with conversions from various error sources:

```rust
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an error occurred with the databse")]
    SQLx(#[from] sqlx::Error),
    #[error("an internal server error occurred")]
    Other(#[from] anyhow::Error),
    #[error("entity not found")]
    NotFound,
    // ... other error variants
}
```

### Secure Password Storage

The project uses Argon2 (a memory-hard hashing algorithm) for secure password storage:

```rust
async fn hash_password(password: &SecretString) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.expose_secret().as_bytes(), &salt)?
        .to_string())
}
```

### Type-Safe Database Queries

SQLx provides compile-time checked SQL queries:

```rust
pub async fn get_all_tasks(pool: &PgPool, user_id: Uuid) -> Result<Vec<Task>> {
    sqlx::query_as!(
        Task,
        r#"
        select * from task
        where user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(Error::SQLx)
}
```

## ⚠️ Educational Notes

This project is designed for learning purposes and demonstrates several advanced concepts:

1. **Middleware Composition**: Learn how to compose multiple middleware layers for cross-cutting concerns like authentication, logging, and error handling.

2. **Error Handling Strategy**: Study the centralized error handling approach for creating consistent user experiences.

3. **Async Programming**: See real-world examples of asynchronous Rust in a web application context.

4. **Security Best Practices**: Learn proper password hashing, session management, and authentication flows.

5. **Database Patterns**: Understand how to organize database interactions in a maintainable way.

## 🔍 Future Learning Opportunities

As you explore this codebase, consider extending it with:

- Rate limiting middleware
- CSRF protection
- Request validation using a validation library
- API documentation with OpenAPI/Swagger
- User roles and permissions
- Metrics collection and monitoring
- Caching strategies
- WebSocket integration
