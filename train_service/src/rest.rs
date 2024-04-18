pub(crate) struct AppState {}

pub(crate) async fn serve(state: AppState) {
    let app = app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("Listening on port 8081");
    axum::serve(listener, app).await.unwrap();
}

fn app(state: AppState) -> axum::Router {
    axum::Router::new()
}
