#!/bin/bash

# This script extracts libpangu.so from the docker image.
# Needed to generate linux versions with ancient Glibc versions.

cd "$(dirname "$0")"
docker build -t pangu ..
docker run pangu cat /target/release/libpangu.so > ./libpangu.so
mkdir -p ../target/linux_old_glibc/
mv libpangu.so ../target/linux_old_glibc/pangu.so
