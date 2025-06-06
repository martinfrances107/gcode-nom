#/usr/bin/bash
rm *.svg
time cargo run --release --  < ../assets/3DBenchy.gcode > benchy.svg
time cargo run --release --  <  ../assets/II\ 2x4_0.15mm_PLA_MK3S_1h17m.gcode  > II.svg
time cargo run --release --  <  ../assets/O\ 6x6_0.15mm_PLA_MK3S_3h56m.gcode > O.svg
time cargo run --release --  <  ../assets/T\ 2x3_0.15mm_PLA_MK3S_1h3m.gcode > T.svg
time cargo run --release --  < ../assets/mini_cube_a.gcode > mini_cube_a.svg
## No StdIn, passed filename as an argument.
time cargo run --release --   ../assets/X\ 6x6_0.15mm_PLA_MK3S_1h55m.gcode > X.svg
# Binary bgcode files must be passed in as an argument because they are not utf-8 encoded.
time cargo run --release -- ../assets/both\ parts.bgcode > both.svg
time cargo run --release -- ../assets/mini_cube_b.bgcode > mini_cube_b.svg
time cargo run --release -- ../assets/benchy2-mk4s.bgcode > benchy2-mk4s.svg
time cargo run --release -- ../assets/gear-holder-print-in-place_04n_022mm_pla_mk4_6h49m.bgcode > gear-holder.svg
# Round box has a bug in the final upper left corner (SVG)
time cargo run --release -- ../assets/round_box.gcode > round_box.svg
# time cargo run -- ../assets/arc_demo.gcode > arc_demo.svg

brave *.svg
