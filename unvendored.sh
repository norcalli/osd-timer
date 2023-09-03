#!/bin/sh
cd "$(dirname "$0")"
vendor() {
	cp .cargo/config.toml{.vendor,}
}
unvendor() {
	cp .cargo/config.toml{.novendor,}
}
unvendor
trap "vendor" EXIT
eval "$@"
