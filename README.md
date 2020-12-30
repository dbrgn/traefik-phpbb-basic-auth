# traefik-phpbb-basic-auth

Log in using static phpbb3 hashes and the Traefik ForwardAuth middleware.

Note: It is not possible to reliably clear basic auth login data stored in the
browser. This means that if a user enters the wrong credentials, the browser
must be closed and re-opened in order for the basic auth login window to pop up
again.

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

- `LOGINS_FILE`: The path to the logins file (defaults to `logins.txt`)
- `BASIC_AUTH_REALM`: The realm used for basic auth (defaults to `Login`)

## Dockerfile

The provided Dockerfile runs the server on port 8080. The data file should be
mounted to `/data/logins.txt`. (An alternative location can be configured with
the `LOGINS_FILE` env var.)

To build:

    docker build . -t dbrgn/traefik-phpbb-basic-auth:latest

To run (with the data in `./logins.txt`):

    docker run --rm -p 8080:8080 -v "$(pwd):/data" dbrgn/traefik-phpbb-basic-auth

To run (with the data in `/tmp/hashes.txt`):

    docker run --rm -p 8080:8080 -v "/tmp:/data" \
        -e LOGINS_FILE=/data/hashes.txt \
        dbrgn/traefik-phpbb-basic-auth

## License

MIT
