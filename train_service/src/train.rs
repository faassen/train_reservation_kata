use ahash::AHashMap as HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct TrainId(String);

impl TrainId {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct SeatId(String);

impl SeatId {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct BookingReference(String);

impl BookingReference {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct TrainDataService {
    trains: HashMap<TrainId, Train>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Train {
    seats: HashMap<SeatId, Seat>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Seat {
    seat_number: String,
    coach: String,
    booking_reference: Option<BookingReference>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Reservation {
    train_id: TrainId,
    seats: Vec<SeatId>,
    booking_reference: BookingReference,
}

#[derive(Debug, PartialEq, Eq)]
enum Error {
    TrainDoesNotExist(TrainId),
    SeatsDoNotExist(Vec<SeatId>),
    SeatsAlreadyReserved(Vec<SeatId>),
}

impl Train {}

impl TrainDataService {
    pub(crate) fn new(trains: HashMap<TrainId, Train>) -> TrainDataService {
        TrainDataService { trains }
    }

    pub(crate) fn train(&self, train_id: &TrainId) -> Result<&Train, Error> {
        self.trains
            .get(train_id)
            .ok_or(Error::TrainDoesNotExist(train_id.clone()))
    }

    pub(crate) fn train_mut(&mut self, train_id: &TrainId) -> Result<&mut Train, Error> {
        self.trains
            .get_mut(train_id)
            .ok_or(Error::TrainDoesNotExist(train_id.clone()))
    }

    pub(crate) fn reserve(&mut self, reservation: &Reservation) -> Result<(), Error> {
        let train = self.train(&reservation.train_id)?;

        // first check whether we have any non-existent seats, report error if any of them are
        let mut non_existent_seat_ids = Vec::new();
        for seat_id in &reservation.seats {
            if !train.seats.contains_key(seat_id) {
                non_existent_seat_ids.push(seat_id.clone());
            }
        }
        if !non_existent_seat_ids.is_empty() {
            return Err(Error::SeatsDoNotExist(non_existent_seat_ids));
        }

        // then report error if any seat is already reserved
        let mut seats_already_reserved = Vec::new();
        for seat_id in &reservation.seats {
            let seat = train.seats.get(seat_id).unwrap();
            if seat.booking_reference.is_some() {
                seats_already_reserved.push(seat_id.clone());
            }
        }
        if !seats_already_reserved.is_empty() {
            return Err(Error::SeatsAlreadyReserved(seats_already_reserved));
        }

        // finally reserve the seats
        let train = self.train_mut(&reservation.train_id)?;
        for seat_id in &reservation.seats {
            let seat = train.seats.get_mut(seat_id).unwrap();
            seat.booking_reference = Some(reservation.booking_reference.clone());
        }

        Ok(())
    }

    pub(crate) fn reset(&mut self, train_id: &TrainId) -> Result<(), Error> {
        todo!();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_train_doesnt_exist() {
        let service = TrainDataService::new(HashMap::new());
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
        let service = TrainDataService::new(trains);
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
        let mut trains = HashMap::new();
        let train = Train {
            seats: HashMap::from([(
                SeatId::new("1A"),
                Seat {
                    seat_number: "1".to_string(),
                    coach: "A".to_string(),
                    booking_reference: None,
                },
            )]),
        };
        let train_id = TrainId::new("foo_train");
        trains.insert(train_id.clone(), train);
        let mut service = TrainDataService::new(trains);
        service
            .reserve(&Reservation {
                train_id: train_id.clone(),
                seats: vec![SeatId::new("1A")],
                booking_reference: BookingReference::new("123456"),
            })
            .unwrap();
        let train = service.train(&train_id).unwrap();
        let seat = train.seats.get(&SeatId::new("1A")).unwrap();
        assert_eq!(
            seat.booking_reference,
            Some(BookingReference::new("123456"))
        );
    }

    #[test]
    fn test_reserve_when_already_reserved() {
        let mut trains = HashMap::new();
        let train = Train {
            seats: HashMap::from([(
                SeatId::new("1A"),
                Seat {
                    seat_number: "1".to_string(),
                    coach: "A".to_string(),
                    booking_reference: Some(BookingReference::new("existing")),
                },
            )]),
        };
        trains.insert(TrainId::new("foo_train"), train);
        let mut service = TrainDataService::new(trains);
        let result = service.reserve(&Reservation {
            train_id: TrainId::new("foo_train"),
            seats: vec![SeatId::new("1A")],
            booking_reference: BookingReference::new("new"),
        });
        assert_eq!(
            result,
            Err(Error::SeatsAlreadyReserved(vec![SeatId::new("1A")]))
        );
    }
}
