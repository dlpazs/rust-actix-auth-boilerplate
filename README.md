# Rust Actix-web Authentication Boilerplate

## Installation

- Requires [Docker](https://docs.docker.com/get-docker/)

```bash
$ docker-compose up -d
```

- Requires [diesel](http://diesel.rs/guides/getting-started/)

```
cargo install diesel_cli --no-default-features --features postgres
```

- Run:

```
diesel setup
diesel migration run
```

```
cargo run
```
