#/usr/bin/bash

rm a.md
RUST_BACKTRACE=1 cargo run  < ../assets/both\ parts.bgcode > a.md
