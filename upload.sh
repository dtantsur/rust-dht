#!/bin/bash

set -eux

HOST=${HOST:-$1}
ROOT=${ROOT:-public_html/rust}

cargo clean
cargo doc --no-deps

ssh $HOST "rm -rf $ROOT"
scp -r target/doc $HOST:$ROOT
ssh $HOST "chmod ag+r -R $ROOT"
ssh $HOST "find $ROOT -type d -exec chmod ag+x \{\} \;"
