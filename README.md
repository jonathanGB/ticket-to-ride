# Ticket To Ride
Web-based version of the "Ticket To Ride" board game, by Alan R. Moon.

## How To Run
The steps below assume you have a recent version of `Rust` (specifically `cargo`), and of `npm`.

1. Compile the front-end.
```bash
$ cd frontend
$ npm i
$ npm run build
```

2. Run the web-server.
```bash
$ cd backend/web-server
$ cargo run
```

By default, the server will run at [http://localhost:8000](http://localhost:8000).