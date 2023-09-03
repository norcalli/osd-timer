#!/bin/sh
cd "$(dirname "$0")"
./unvendored.sh cargo vendor --no-delete
