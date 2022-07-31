# The Ticket To Ride binary!

This crate contains all the modules to run the *Ticket To Ride* server, which closely depends on the [ticket-to-ride](../ticket-to-ride/) library crate for all the game-specific logic -- more specifically, the game manager.

The server is launched in [main.rs](src/main.rs), the routes handlers are defined in [router.rs](src/router.rs), and the glue between authentication, player actions, and game logic is in [controller.rs](src/controller.rs).

## Documentation

To generate documentation for this crate, run the following command (from the current directory):

```bash
$ cargo doc --open
```