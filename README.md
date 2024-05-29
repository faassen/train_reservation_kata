# Kata: Train Reservation

Railway operators aren't always known for their use of cutting edge technology,
and in this case they're a little behind the times. The railway people want you
to help them to improve their online booking service. They'd like to be able to
not only sell tickets online, but to decide exactly which seats should be
reserved, at the time of booking.

You're working on the Ticket Office service, and your next task is to implement
the feature for reserving seats on a particular train. The railway operator has
a service-oriented architecture, and both the interface you'll need to fulfill,
and some services you'll need to use, are already implemented.

## Business Rules around Reservations

There are various business rules and policies around which seats may be
reserved. For a train overall, no more than 70% of seats may be reserved in
advance, and ideally no individual coach should have no more than 70% reserved
seats either. However, there is another business rule that says you must put
all the seats for one reservation in the same coach. This could make you go
over 70% for some coaches, just make sure to keep to 70% for the whole train.

## The new service to build

The Ticket Office service needs you to implement a REST service on the
`/reserve` URL. It should respond to a HTTP POST request for a train that comes
with JSON data in the request body describing which train the customer wants to
reserve seats on, and how many seats they want:

```json
{
  "train_id": "express_2000",
  "seat_count": 4
}
```

In the request body return a JSON document detailing the reservation that has been
made:

```json
{
  "train_id": "express_2000",
  "booking_reference": "75bcd15",
  "seats": ["1A", "2A", "3A", "4A"]
}
```

If it is not possible to find suitable seats to reserve, the service should
instead return an empty list of seats and `null` for the booking
reference:

```json
{
  "train_id": "express_2000",
  "booking_reference": null,
  "seats": []
}
```

To implement this behavior, you can use the backend REST services described
below. Be careful to implement the business rules!

### Command line option

As an alternative to coming up with a fully deployed HTTP service, you can
instead write a command line program which takes the train id and number of
seats as command line arguments, and returns the same reservation JSON as above.

## Train services

Some backend REST APIs have already been implemented that you need to use in
the implementation. The following endpoints are provided:

- `/booking_reference` to create a new booking reference.

- `/train/<train_id>` to get information about a train.

- `/train/train_id>/reserve` to reserve seats on the train.

- `/train/<train_id>/reset` to reset reservations in a train.

For testing purposes, there is a local service you can run locally. You can
assume the real service will behave the same way, but be available on a
different URL.

Start the server by going into the `train_service` directory and running `cargo :

```bash
cargo run
```

It will start on http://localhost:8081.

Let's go into each endpoint in detail.

### Booking Reference Endpoint at `/booking_reference`

You can use this service to get a unique booking reference. Make a `POST`
request to:

```
http://localhost:8081/booking_reference
```

This will return a string that looks a bit like this:

```
75bcd15
```

### Train Data Endpoint at `/train/<train_id>`

You can get information about which seats each train has by using the train
data service. This is available under `/train/<train_id>`.

There are two trains available with ids `local_1000` and `express_2000`.

For example, you can send a GET request to `/train/express_2000` to get data about
the `express_2000` train:

```
/train/express_2000
```

This returns a JSON document with information about the seats that this train
has available, with a structure like this:

```json
{
  "seats": {
    "1A": { "booking_reference": null, "seat_number": "1", "coach": "A" },
    "2A": { "booking_reference": null, "seat_number": "2", "coach": "A" }
  }
}
```

Note that we've left out all the extraneous details about where the train is
going to and from, at what time, whether there's a buffet car etc. All that's
there is which seats the train has, and if they are already booked. A seat is
available if the `booking_reference` field contains `null`.

### Reservation Endpoint

To reserve seats on a train, you'll need to make a `POST` request to this URL:

```
/train/<train_id>/reserve
```

with as the body a JSON document for which seats to reserve. There should be
two fields:

```json
{
  "booking_reference": "75bcd15",
  "seats": ["1A", "2A"]
}
```

Note that the server will prevent you from booking non-existent seats, as well
as seats that are already reserved with another booking reference.

### Reset endpoint

The service has one additional method, that will remove all reservations on a
particular train. Use it with care:

```
/train/<train_id>/reset`
```

## Credits

Based off [Emily Bache's version of this
Kata](https://github.com/emilybache/KataTrainReservation) but with services in
Rust. I formulated the REST API differently. All data exchange uses JSON,
without the use of HTTP forms.
