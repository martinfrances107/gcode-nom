#/usr/bin/bash
rm *.obj
time cargo run --release -- -a < ../assets/3DBenchy.gcode > benchy.obj
time cargo run --release -- -a <  ../assets/II\ 2x4_0.15mm_PLA_MK3S_1h17m.gcode  > II.obj
time cargo run --release -- -a <  ../assets/O\ 6x6_0.15mm_PLA_MK3S_3h56m.gcode > O.obj
time cargo run --release -- -a <  ../assets/T\ 2x3_0.15mm_PLA_MK3S_1h3m.gcode > T.obj
time cargo run --release --  < ../assets/mini_cube_a.gcode > mini_cube_a.obj
# Not using stdin, passing file path as an argument.
time cargo run --release -- -a ../assets/X\ 6x6_0.15mm_PLA_MK3S_1h55m.gcode > X.obj
# Binary bgcoode files must be passed in as an argument because they are not utf-8 encoded.
time cargo run --release -- ../assets/both\ parts.bgcode > both.obj
time cargo run --release -- ../assets/mini_cube_b.bgcode > mini_cube_b.obj
time cargo run --release -- ../assets/benchy2-mk4s.bgcode > benchy2-mk4s.obj

# Performance: This takes 1m 20s to render!!!!
time cargo run --release -- ../assets/gear-holder-print-in-place_04n_022mm_pla_mk4_6h49m.bgcode > gear-holder.obj