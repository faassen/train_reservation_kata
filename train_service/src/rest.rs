use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::Router;

use crate::booking_reference::{BookingReference, BookingReferenceService};
use crate::train::{self, TrainDataService, TrainId};

pub(crate) struct AppState {
    booking_reference_service: BookingReferenceService,
    train_data_service: TrainDataService,
}

impl AppState {
    pub(crate) fn new() -> AppState {
        let trains_str = include_str!("trains.json");
        let trains = serde_json::from_str(trains_str).unwrap();
        AppState {
            booking_reference_service: BookingReferenceService::new(0),
            train_data_service: TrainDataService::new(trains),
        }
    }
}

pub(crate) async fn serve(state: AppState) {
    let app = app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("Listening on port 8081");
    axum::serve(listener, app).await.unwrap();
}

fn app(state: AppState) -> axum::Router {
    let state = Arc::new(Mutex::new(state));
    axum::Router::new()
        .route(
            "/booking_reference",
            post(booking_reference).with_state(state.clone()),
        )
        .route("/train/:train_id", get(train).with_state(state.clone()))
}

async fn booking_reference(
    State(state): State<Arc<Mutex<AppState>>>,
) -> axum::Json<BookingReference> {
    let reference = state
        .lock()
        .unwrap()
        .borrow_mut()
        .booking_reference_service
        .booking_reference();
    axum::Json(reference)
}

async fn train(
    Path(train_id): Path<TrainId>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> axum::Json<train::Train> {
    let train = state
        .lock()
        .unwrap()
        .borrow_mut()
        .train_data_service
        .train(&train_id)
        .unwrap()
        .clone();
    axum::Json(train)
}

#[cfg(test)]
mod tests {
    use axum_test::{TestServer, TestServerConfig};

    use crate::train::{Train, TrainId, TrainsData};

    // based around https://github.com/tokio-rs/axum/blob/main/examples/testing/src/main.rs
    use super::*;

    fn new_test_app() -> TestServer {
        let app = app(AppState::new());
        let config = TestServerConfig::builder()
            .expect_success_by_default()
            .mock_transport()
            .build();

        TestServer::new_with_config(app, config).unwrap()
    }

    #[tokio::test]
    async fn test_booking_reference() {
        let server = new_test_app();

        let response = server
            .post("/booking_reference")
            .await
            .json::<BookingReference>();

        assert_eq!(response, BookingReference::new("1"));
    }

    #[tokio::test]
    async fn test_train_local_2000_get() {
        let server = new_test_app();

        let train = server.get("/train/local_1000").await.json::<Train>();

        let trains_str = include_str!("trains.json");
        let trains: TrainsData = serde_json::from_str(trains_str).unwrap();
        let local_2000 = trains.get(&TrainId::new("local_1000")).unwrap();

        assert_eq!(&train, local_2000);
    }
}
