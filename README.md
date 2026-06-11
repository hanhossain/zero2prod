# zero2prod

## Local deployment

```shell
docker compose up --wait
cargo sqlx database setup
```

## Other useful commands

- `cargo sqlx prepare -- --all-targets` - prepare sqlx offline metadata
- `cargo sqlx migrate add <migration name>` - generate migration
