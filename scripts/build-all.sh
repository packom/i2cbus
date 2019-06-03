#!/bin/bash
set -e

docker login -u packom
scripts/build-container.sh arm
scripts/build-container.sh armv7
scripts/build-container.sh x86_64
docker push packom/i2cbus-arm:0.1.1
docker push packom/i2cbus-armv7:0.1.1
docker push packom/i2cbus-x86_64:0.1.1
