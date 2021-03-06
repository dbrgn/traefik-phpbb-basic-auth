FROM rust:1 AS builder

RUN rustup target add x86_64-unknown-linux-musl

COPY . /code

RUN cd /code \
&&  cargo build --release --target x86_64-unknown-linux-musl


FROM alpine:3.12

RUN apk update && apk add dumb-init

RUN mkdir /data/ \
&&  addgroup -S www \
&&  adduser -S -G www www \
&&  chown www:www /data/ \
&&  chmod 0700 /data/

VOLUME [ "/service" ]

COPY --from=builder /code/target/x86_64-unknown-linux-musl/release/traefik-phpbb-basic-auth /usr/local/bin/traefik-phpbb-basic-auth

WORKDIR /data

USER www

# Note: Use dumb-init in order to fulfil our PID 1 responsibilities,
# see https://github.com/Yelp/dumb-init
ENTRYPOINT [ "/usr/bin/dumb-init", "--" ]
CMD [ "traefik-phpbb-basic-auth" ]
