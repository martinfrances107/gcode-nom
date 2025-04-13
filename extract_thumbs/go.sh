rm *.qoi
rm *.jpg
rm *.png

# Performance: This takes 0.078s to render.
time cargo run --release -- ../assets/gear-holder-print-in-place_04n_022mm_pla_mk4_6h49m.bgcode

