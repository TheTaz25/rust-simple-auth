# PREREQUISITES
## DIESEL
Before installing diesel_cli, you need the following libraries:
1. libpq
2. mysql-client

Installing diesel_cli can be done with:
`RUSTFLAGS='-L /opt/homebrew/opt/libpq/lib' cargo install diesel_cli`
be aware to explicitly specify the libpq location via RUSTFLAGS as homebrew does provide the package keg-only
If necessary, you might need to add rust binaries to your path, you will be notified by cargo if necessary.

### Runnung Migrations with DIESEL_CLI
`diesel migration run --database-url postgres://<USER>:<PASSWORD>@<HOST>/<DB_NAME>`