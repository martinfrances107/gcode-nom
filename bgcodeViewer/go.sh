#/usr/bin/bash

rm a.md
time RUST_LOG=trace RUST_BACKTRACE=1 cargo run  < ../assets/both\ parts.bgcode > a.md
