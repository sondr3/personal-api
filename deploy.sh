#!/usr/bin/env bash

echo "Redeploying personal API!"

echo "Getting version"
version=$(awk -F'[ ="]+' '$1 == "version" { print $2 }' Cargo.toml)

echo "Building new docker container..."
docker build --build-arg version=v"$version" -t personal-api . | cat

echo "Stopping service..."
docker stop personal-api

echo "Removing old container..."
docker container rm personal-api

echo "Redeploying service"
docker run -itd --restart unless-stopped -p 8000:8081 --name personal-api personal-api

echo "And we're live again!"