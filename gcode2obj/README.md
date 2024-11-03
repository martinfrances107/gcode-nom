# gcodeToObj

Rust 2021 Edition.

A G-code visualization tool written in [rust](https://www.rust-lang.org/)

A nom based parser, outputs a "Wavefront Obj" file which can be imported into blender and a Bevy app for visuallization

## How to use

parses StdIn as a gcode file - the SVG file is send to StdOut :-

```bash
cargo run --release -- < ../assets/bency.gcode > benchy.obj
```

see [todo](TODO.md)

## Future work

What to do when nozzle is lifted up and a new part started.
