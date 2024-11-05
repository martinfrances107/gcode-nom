#/usr/bin/bash

rm a.svg
time cargo run --release --  < ../assets/3DBenchy.gcode > a.svg
brave a.svg