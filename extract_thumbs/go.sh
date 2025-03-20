#time  RUST_BACKTRACE=1 cargo run  --  ../assets/mini_cube_b.bgcode
rm *.oci
rm *.png
# time RUST_LOG=trace cargo run --release  --  ../assets/both\ parts.bgcode

# Performance: This takes 1m 20s to render!!!!
cargo run --release -- ../assets/gear-holder-print-in-place_04n_022mm_pla_mk4_6h49m.bgcode

