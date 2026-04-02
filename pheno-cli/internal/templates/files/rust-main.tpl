use axum::serve;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use {{.RepoName}}::adapters::web::create_routes;
use {{.RepoName}}::adapters::persistence::InMemoryEntityRepository;
use {{.RepoName}}::application::services::EntityServiceImpl;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let repository = InMemoryEntityRepository::new();
    let service = EntityServiceImpl::new(repository);

    let app = create_routes(service);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    
    tracing::info!("Server starting on {}", addr);
    
    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app).await.unwrap();
}
