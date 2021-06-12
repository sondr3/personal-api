#!/usr/bin/env bash

echo "Redeploying personal API!"

echo "Getting version"
VERSION=$(awk -F'[ ="]+' '$1 == "version" { print $2 }' Cargo.toml)

echo "Redeploying service"
API_VERSION=${VERSION} docker-compose up -d --build

echo "And we're live again!"
