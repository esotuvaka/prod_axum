## Server (Axum)

### Start the server with hot reloading (development)

> NOTE: Install cargo watch with `cargo install cargo-watch`

```bash
cargo watch -q -c -w src/ -w .cargo/ -x "run"
```

### Start the server in production mode

```bash
cargo run --release
```

## Database (Postgres)

### Start the DB

```bash
docker run --rm --name pg -p 5432:5432 -e POSTGRES_PASSWORD=welcome postgres:15
```

### Create Postgres terminal

```bash
docker exec -it -u postgres pg psql
```

## Unit Test

> NOTE: Install cargo watch with `cargo install cargo-watch`.

```bash
cargo watch -q -c -x "test -- --nocapture"

# Specify test with a filter
cargo watch -q -c -x "test model::task::tests::test_create -- --nocapture"
```
