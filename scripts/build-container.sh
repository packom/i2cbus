#!/bin/bash
set -e

function output_args {
    echo "Usage: build-container.sh <arch>"
    echo "  <arch> = x86_64|arm|armv7"
    exit 1
}

ARCH=$1
if [[ ! $ARCH ]];
  then
    output_args
fi

if [[ $ARCH == "x86_64" ]];
  then
    TARGET="x86_64-unknown-linux-musl"
elif [[ $ARCH == "arm" ]]
  then
    TARGET="arm-unknown-linux-musleabihf"
elif [[ $ARCH == "armv7" ]]
  then
    TARGET="armv7-unknown-linux-musleabihf"
else
  output_args
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

DIR=tmp/$BIN-$ARCH-$VERSION
TAG=$BIN-$ARCH:$VERSION

echo "Creating container for"
echo "  Binary:   $BIN"
echo "  Arch:     $ARCH"
echo "  Target:   $TARGET"
echo "  Version:  $VERSION"
echo "  Tag:      $TAG"

rm -fr $DIR
mkdir -p $DIR
cp target/$TARGET/release/$BIN $DIR/
cp ../$BIN-api/api/openapi.yaml $DIR/api.yaml
echo "cargo build --release --target $TARGET"
cargo build --release --target $TARGET
echo "docker build -t $TAG -f scripts/Dockerfile $DIR"
docker build -t $TAG -f scripts/Dockerfile $DIR
rm -fr $DIR
