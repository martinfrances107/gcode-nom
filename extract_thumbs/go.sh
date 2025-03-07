#time  RUST_BACKTRACE=1 cargo run  --  ../assets/mini_cube_b.bgcode
rm *.oci
rm *.png
time RUST_BACKTRACE=1 cargo run --release  --  ../assets/both\ parts.bgcode

