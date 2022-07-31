# Backend

## Crates

There are two main components (crates in Rust jargon) for the backend.

1. The game logic (in [ticket-to-ride/](ticket-to-ride/)): this library crate handles game creation, transitions across game states, and persisting the state of a game.
2. The Web server (in [web-server/](web-server/)): this binary crate is in charge of running the server (using the [Rocket framework](https://rocket.rs)), and providing HTTP endpoints for web clients. It closely depends on the game logic library.
