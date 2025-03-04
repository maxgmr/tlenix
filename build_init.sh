#!/usr/bin/env bash

clang -Wall -std=c23 -nostdlib -ffreestanding -no-pie init/init.c init/start.S -o init/init
