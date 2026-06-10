# zero2prod

## Local deployment

```shell
docker compose up --wait
sqlx database setup
```

## Other useful commands

- `cargo sqlx prepare -- --all-targets` - prepare sqlx offline metadata
