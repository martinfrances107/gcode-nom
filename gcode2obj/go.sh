#/usr/bin/bash

time cargo run --release -- -a < ../assets/3DBenchy.gcode > benchy.obj
time cargo run --release -- -a <  ../assets/II\ 2x4_0.15mm_PLA_MK3S_1h17m.gcode  > II.obj
time cargo run --release -- -a <  ../assets/O\ 6x6_0.15mm_PLA_MK3S_3h56m.gcode > O.obj
time cargo run --release -- -a <  ../assets/T\ 2x3_0.15mm_PLA_MK3S_1h3m.gcode > T.obj
time cargo run --release -- -a <  ../assets/X\ 6x6_0.15mm_PLA_MK3S_1h55m.gcode > X.obj