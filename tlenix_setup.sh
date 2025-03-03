#!/usr/bin/env bash

if [ -z "$(ls -A 'kernel_build')" ]; then
    echo "please run kernel_build.sh to build the kernel before running this script"
    exit 1
fi
