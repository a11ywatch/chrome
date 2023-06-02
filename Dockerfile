FROM rust:alpine3.17 AS rustbuilder

WORKDIR /app

RUN apk upgrade --update-cache --available && \
	apk add gcc cmake make g++

COPY . .

RUN cargo install --no-default-features --path .

FROM zenika/alpine-chrome

EXPOSE 9222 6000

USER root

COPY --from=rustbuilder /usr/local/cargo/bin/chrome_driver /usr/local/bin/chrome_driver
COPY ./docker-entrypoint.sh /

RUN apk add --no-cache tini curl dbus
RUN chmod +x /docker-entrypoint.sh
RUN mkdir -p /var/run/dbus

USER chrome

ENV REMOTE_ADDRESS=0.0.0.0

ENTRYPOINT ["tini", "--", "/docker-entrypoint.sh"]
