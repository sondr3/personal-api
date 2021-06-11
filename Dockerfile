FROM debian:buster-slim

ARG version=v0.1.0

RUN apt-get update && apt-get install wget -y

WORKDIR /usr/src/api

RUN wget https://github.com/sondr3/personal-api/releases/download/${version}/personal-api-x86_64-unknown-linux-musl.tar.gz

RUN tar xvf personal-api-x86_64-unknown-linux-musl.tar.gz

CMD ["./personal-api"]