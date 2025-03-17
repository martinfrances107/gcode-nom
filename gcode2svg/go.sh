#/usr/bin/bash
rm *.svg
time cargo run --release --  < ../assets/3DBenchy.gcode > benchy.svg
time cargo run --release --  <  ../assets/II\ 2x4_0.15mm_PLA_MK3S_1h17m.gcode  > II.svg
time cargo run --release --  <  ../assets/O\ 6x6_0.15mm_PLA_MK3S_3h56m.gcode > O.svg
time cargo run --release --  <  ../assets/T\ 2x3_0.15mm_PLA_MK3S_1h3m.gcode > T.svg
## No StdIn, passed filename as an argument.
time cargo run --release --   ../assets/X\ 6x6_0.15mm_PLA_MK3S_1h55m.gcode > X.svg
## Binary bgcoode files must be passed in as an argument beacuse they are not utf-8 encoded.
time RUST_LOG=trace cargo run --release --  ../assets/both\ parts.bgcode > both.svg
brave *.svg
