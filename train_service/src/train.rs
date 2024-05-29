use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use crate::booking_reference::BookingReference;

#[derive(Debug, PartialEq, Eq, Clone, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) struct TrainId(String);

impl TrainId {
    pub(crate) fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }
}

impl Display for TrainId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) struct SeatId(String);

impl SeatId {
    pub(crate) fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }
}

impl Display for SeatId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct TrainDataService {
    trains: TrainsData,
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct TrainsData(HashMap<TrainId, Train>);

impl TrainsData {
    fn new() -> Self {
        TrainsData(HashMap::new())
    }

    pub(crate) fn get(&self, train_id: &TrainId) -> Option<&Train> {
        self.0.get(train_id)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Train {
    seats: HashMap<SeatId, Seat>,
}

impl Train {
    pub(crate) fn get(&self, seat_id: &SeatId) -> Option<&Seat> {
        self.seats.get(seat_id)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Seat {
    seat_number: String,
    coach: String,
    booking_reference: Option<BookingReference>,
}

impl Seat {
    pub(crate) fn booking_reference(&self) -> Option<&BookingReference> {
        self.booking_reference.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Reservation {
    pub(crate) seats: Vec<SeatId>,
    pub(crate) booking_reference: BookingReference,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Error {
    TrainDoesNotExist(TrainId),
    SeatsDoNotExist(Vec<SeatId>),
    SeatsAlreadyReserved(Vec<SeatId>),
}

impl Train {
    pub(crate) fn reserve(&mut self, reservation: &Reservation) -> Result<(), Error> {
        // first check whether we have any non-existent seats, report error if any of them are
        let mut non_existent_seat_ids = Vec::new();
        for seat_id in &reservation.seats {
            if !self.seats.contains_key(seat_id) {
                non_existent_seat_ids.push(seat_id.clone());
            }
        }
        if !non_existent_seat_ids.is_empty() {
            return Err(Error::SeatsDoNotExist(non_existent_seat_ids));
        }

        // then report error if any seat is already reserved
        let mut seats_already_reserved = Vec::new();
        for seat_id in &reservation.seats {
            let seat = self.seats.get(seat_id).unwrap();
            if seat.booking_reference.is_some() {
                seats_already_reserved.push(seat_id.clone());
            }
        }

        if !seats_already_reserved.is_empty() {
            return Err(Error::SeatsAlreadyReserved(seats_already_reserved));
        }

        // finally reserve the seats
        for seat_id in &reservation.seats {
            let seat = self.seats.get_mut(seat_id).unwrap();
            seat.booking_reference = Some(reservation.booking_reference.clone());
        }

        Ok(())
    }

    pub(crate) fn reset(&mut self) {
        for seat in self.seats.values_mut() {
            seat.booking_reference = None;
        }
    }
}

impl TrainDataService {
    pub(crate) fn new(trains: TrainsData) -> TrainDataService {
        TrainDataService { trains }
    }

    pub(crate) fn train(&self, train_id: &TrainId) -> Result<&Train, Error> {
        self.trains
            .0
            .get(train_id)
            .ok_or(Error::TrainDoesNotExist(train_id.clone()))
    }

    pub(crate) fn train_mut(&mut self, train_id: &TrainId) -> Result<&mut Train, Error> {
        self.trains
            .0
            .get_mut(train_id)
            .ok_or(Error::TrainDoesNotExist(train_id.clone()))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_train_doesnt_exist() {
        let service = TrainDataService::new(TrainsData::new());
        let train_id = TrainId::new("doesnt_exist");
        let train = service.train(&train_id).unwrap_err();
        assert_eq!(train, Error::TrainDoesNotExist(train_id));
    }

    #[test]
    fn test_train_does_exist() {
        let mut trains = HashMap::new();
        let train = Train {
            seats: HashMap::from([(
                SeatId::new("1A"),
                Seat {
                    seat_number: "1".to_string(),
                    coach: "A".to_string(),
                    booking_reference: Some(BookingReference::new("123456")),
                },
            )]),
        };
        let train_id = TrainId::new("train_id");
        trains.insert(train_id.clone(), train);
        let service = TrainDataService::new(TrainsData(trains));
        let train = service.train(&train_id).unwrap();
        assert_eq!(
            train,
            &Train {
                seats: HashMap::from([(
                    SeatId::new("1A"),
                    Seat {
                        seat_number: "1".to_string(),
                        coach: "A".to_string(),
                        booking_reference: Some(BookingReference::new("123456")),
                    },
                )]),
            }
        );
    }

    #[test]
    fn test_reserve_seat() {
        let mut train = Train {
            seats: HashMap::from([(
                SeatId::new("1A"),
                Seat {
                    seat_number: "1".to_string(),
                    coach: "A".to_string(),
                    booking_reference: None,
                },
            )]),
        };
        train
            .reserve(&Reservation {
                seats: vec![SeatId::new("1A")],
                booking_reference: BookingReference::new("123456"),
            })
            .unwrap();
        let seat = train.seats.get(&SeatId::new("1A")).unwrap();
        assert_eq!(
            seat.booking_reference,
            Some(BookingReference::new("123456"))
        );
    }

    #[test]
    fn test_reserve_when_already_reserved() {
        let mut train = Train {
            seats: HashMap::from([(
                SeatId::new("1A"),
                Seat {
                    seat_number: "1".to_string(),
                    coach: "A".to_string(),
                    booking_reference: Some(BookingReference::new("existing")),
                },
            )]),
        };
        let result = train.reserve(&Reservation {
            seats: vec![SeatId::new("1A")],
            booking_reference: BookingReference::new("new"),
        });
        assert_eq!(
            result,
            Err(Error::SeatsAlreadyReserved(vec![SeatId::new("1A")]))
        );
    }
}
