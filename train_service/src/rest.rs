use std::borrow::{Borrow, BorrowMut};
use std::sync::{Arc, Mutex};

use axum::extract;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};

use crate::booking_reference::BookingReferenceService;
use crate::train::{Error, Reservation, SeatId, TrainDataService, TrainId};

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
        .route(
            "/train/:train_id/reserve",
            post(train_reserve).with_state(state.clone()),
        )
}

async fn booking_reference(
    extract::State(state): extract::State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    let reference = state
        .lock()
        .unwrap()
        .borrow_mut()
        .booking_reference_service
        .booking_reference();
    axum::Json(reference)
}

async fn train(
    extract::Path(train_id): extract::Path<TrainId>,
    extract::State(state): extract::State<Arc<Mutex<AppState>>>,
) -> Result<impl IntoResponse, Error> {
    let train = state
        .lock()
        .unwrap()
        .borrow()
        .train_data_service
        .train(&train_id)?
        .clone();
    Ok(axum::Json(train))
}

async fn train_reserve(
    extract::Path(train_id): extract::Path<TrainId>,
    extract::State(state): extract::State<Arc<Mutex<AppState>>>,
    extract::Json(reservation): extract::Json<Reservation>,
) -> Result<impl IntoResponse, Error> {
    let mut state = state.lock().unwrap();
    let train = state.borrow_mut().train_data_service.train_mut(&train_id)?;
    train.reserve(&reservation)?;
    Ok(axum::Json(train.clone()))
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::TrainDoesNotExist(train_id) => (
                StatusCode::NOT_FOUND,
                format!("Train {} does not exist", train_id),
            )
                .into_response(),
            Error::SeatsAlreadyReserved(seats) => (
                StatusCode::BAD_REQUEST,
                format!("Seats [{}] are already reserved", format_seat_ids(&seats)),
            )
                .into_response(),
            Error::SeatsDoNotExist(seats) => (
                StatusCode::BAD_REQUEST,
                format!("Seats [{}] do not exist", format_seat_ids(&seats)),
            )
                .into_response(),
        }
    }
}

fn format_seat_ids(seats: &[SeatId]) -> String {
    seats
        .iter()
        .map(|seat_id| seat_id.to_string())
        .collect::<Vec<String>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use axum_test::{TestServer, TestServerConfig};

    use crate::{
        booking_reference::BookingReference,
        train::{SeatId, Train, TrainId, TrainsData},
    };

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

    fn new_test_app_failing() -> TestServer {
        let app = app(AppState::new());
        let config = TestServerConfig::builder().mock_transport().build();
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
    async fn test_train_local_1000_get() {
        let server = new_test_app();

        let train = server.get("/train/local_1000").await.json::<Train>();

        let trains_str = include_str!("trains.json");
        let trains: TrainsData = serde_json::from_str(trains_str).unwrap();
        let local_2000 = trains.get(&TrainId::new("local_1000")).unwrap();

        assert_eq!(&train, local_2000);
    }

    #[tokio::test]
    async fn test_train_does_not_exist() {
        let server = new_test_app_failing();

        let response = server.get("/train/does_not_exist").await.status_code();

        assert_eq!(response, 404);
    }

    #[tokio::test]
    async fn test_reserve() {
        let server = new_test_app();

        let train = server
            .post("/train/local_1000/reserve")
            .json(&Reservation {
                seats: vec![SeatId::new("1A")],
                booking_reference: BookingReference::new("123456"),
            })
            .await
            .json::<Train>();

        assert_eq!(
            train
                .get(&SeatId::new("1A"))
                .unwrap()
                .booking_reference()
                .unwrap(),
            &BookingReference::new("123456")
        );
    }

    #[tokio::test]
    async fn test_reserve_seat_does_not_exist() {
        let server = new_test_app_failing();

        let response = server
            .post("/train/local_1000/reserve")
            .json(&Reservation {
                seats: vec![SeatId::new("does_not_exist")],
                booking_reference: BookingReference::new("123456"),
            })
            .await;

        assert_eq!(response.status_code(), 400);
        assert_eq!(response.text(), "Seats [does_not_exist] do not exist");
    }

    #[tokio::test]
    async fn test_reserve_seat_already_reserved() {
        let server = new_test_app_failing();

        // make a reservation
        server
            .post("/train/local_1000/reserve")
            .json(&Reservation {
                seats: vec![SeatId::new("1A")],
                booking_reference: BookingReference::new("123456"),
            })
            .await;

        // try to reserve again
        let response = server
            .post("/train/local_1000/reserve")
            .json(&Reservation {
                seats: vec![SeatId::new("1A")],
                booking_reference: BookingReference::new("123456"),
            })
            .await;

        assert_eq!(response.status_code(), 400);
        assert_eq!(response.text(), "Seats [1A] are already reserved");
    }
}
