#!/bin/bash
set -e

function output_args {
    echo "Usage: run.sh <arch> [port]"
    echo "  <arch> = x86_64|arm|armv7"
    exit 1
}

ARCH=$1
if [[ ! $ARCH ]];
  then
    output_args
fi

PORT=$2
if [[ ! $PORT ]];
  then
    PORT=8080
fi

VERSION="$(awk '/^version = /{print $3}' Cargo.toml | sed 's/"//g' | sed 's/\r$//')"
if [[ ! $VERSION ]];
  then
    echo "Couldn't get version from Cargo.toml"
    exit 1
fi

BIN="$(awk '/^name = /{print $3}' Cargo.toml | sed 's/"//g' | sed 's/\r$//')"
if [[ ! $BIN ]];
  then
    echo "Couldn't get binary from Cargo.toml"
    exit 1
fi

TAG=$BIN-$ARCH:$VERSION
NAME=$BIN-$ARCH

echo "Running container"
echo "  Tag:  $TAG"
echo "  Port: $PORT"
echo "  Name: $NAME"
echo "docker run --name $NAME -d -e GFA_I2CBUS_IP=pi-esp32 -e GFA_I2CBUS_PORT=8080 -p $PORT:8080 $TAG"
docker run --name $NAME -d --device /dev/i2c-1:/dev/i2c-1 --group-add 998 -p $PORT:8080 $TAG