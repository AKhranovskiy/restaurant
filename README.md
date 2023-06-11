# Restaurant ordering system

## Backend

Backend runs on `axum` and stores data in the in-memory SQLite DB with `sqlx`.

Provided endpoints:

  * `GET /meals` returns the list of meals in the menu.
  * `PUT /table/:table/meal/:meal` puts a new order for `:meal` on `:table`.
  * `GET /table/:table/orders` returns all active orders for `:table`.
  * `GET /order/:order` returns an `:order`.
  * `DELETE /order/:order` deletes an `:order`.

### Testing

Run unit tests for the backend

```shell
cargo test
```

## Server app

The Server app runs the backend on `0.0.0.0:9000`.

```shell
cargo run --release --bin server
```

## Clients app

The clients app simulates team of waiters serving pool of tables.

On each iteration, a new table is selected from the pool.
Then the state of the table is updated
with some chance (`EMPTY -> ORDERING <-> EATING -> COMPLETE -> EMPTY`).

A waiter takes a table from the pool of tables and serves according to a table state, then pushes the table back to pool.

### Running

Start the server in the first terminal. Then start the clients app in the second terminal

```shell
cargo run --release --bin clients
```
