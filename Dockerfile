FROM debian:buster-slim

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8081
ENV ROCKET_PROFILE=release

EXPOSE 8081

ARG version=v0.1.0

RUN apt-get update && apt-get install wget -y

WORKDIR /app

RUN wget https://github.com/sondr3/personal-api/releases/download/${version}/personal-api-x86_64-unknown-linux-musl.tar.gz

RUN tar xvf personal-api-x86_64-unknown-linux-musl.tar.gz

COPY .env .

CMD ["./personal-api"]
