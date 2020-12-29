# traefik-phpbb-basic-auth

Log in using static phpbb3 hashes and the Traefik ForwardAuth middleware.

## Usage

    traefik-phpbb-basic-auth <hashes-file>

The hashes file must contain username and password hash separated by a
semicolon, one credentials pair per line. There should be no quoting or CSV
header.

## License

MIT
