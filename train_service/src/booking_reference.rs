pub(crate) struct BookingReferenceService {
    counter: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct BookingReference(String);

impl BookingReference {
    pub(crate) fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }
}

impl BookingReferenceService {
    pub(crate) fn new(start: u64) -> Self {
        BookingReferenceService { counter: start }
    }

    pub(crate) fn booking_reference(&mut self) -> BookingReference {
        self.counter += 1;
        // return a hex number
        BookingReference::new(format!("{:x}", self.counter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_booking_number_looks_like_a_suitable_string() {
        let mut service = BookingReferenceService::new(123456789);
        let booking_reference = service.booking_reference();
        assert_eq!(booking_reference, BookingReference::new("75bcd16"));
    }

    #[test]
    fn test_booking_number_is_unique() {
        let mut service = BookingReferenceService::new(123456789);
        let booking_reference1 = service.booking_reference();
        let booking_reference2 = service.booking_reference();
        assert_ne!(booking_reference1, booking_reference2);
    }
}
