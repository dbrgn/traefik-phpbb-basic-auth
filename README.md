# traefik-phpbb-basic-auth

Log in using static phpbb3 hashes and the Traefik ForwardAuth middleware.

## Building

Requirements: Rust and Cargo.

    cargo build --release

## Usage

    traefik-phpbb-basic-auth <hashes-file>

The hashes file must contain username and password hash separated by a
semicolon, one credentials pair per line. There should be no quoting or CSV
header.

## Configuration

The service can be configured using the following env vars:

- `BASIC_AUTH_REALM`: The realm used for basic auth (defaults to `Login`)

## Dockerfile

The provided Dockerfile runs the server on port 8080. The data file should be
mounted to `/data/logins.csv`. (An alternative file name can be passed in as
argument.)

To build:

    docker build . -t dbrgn/traefik-phpbb-basic-auth:latest

To run (with the data in `logins.csv`):

    docker run --rm -p 8080:8080 -v "$(pwd):/data" dbrgn/traefik-phpbb-basic-auth

To run (with the data in `/tmp/hashes.csv`):

    docker run --rm -p 8080:8080 -v "/tmp:/data" dbrgn/traefik-phpbb-basic-auth /data/hashes.csv

## License

MIT
