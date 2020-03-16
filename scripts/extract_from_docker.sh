#!/bin/bash

# This script extracts libpangu.so from the docker image.
# Needed to generate linux versions with ancient Glibc versions.

docker create -ti --name dummy januz/pangu
docker cp dummy:/target/release/libpangu.so .
docker rm -fv dummy
mkdir -p ../target/linux_old_glibc/
mv libpangu.so ../target/linux_old_glibc/pangu.so
