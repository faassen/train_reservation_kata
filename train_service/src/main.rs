mod booking_reference;
mod rest;
mod train;

use rest::serve;

#[tokio::main]
async fn main() {
    let app_state = rest::AppState::new();
    serve(app_state).await
}
