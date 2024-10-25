## Server (Axum)

### Start the server with hot reloading (development)

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
