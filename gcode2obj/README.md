# gcodeToObj

Rust 2021 Edition.

A G-code visualization tool written in [rust](https://www.rust-lang.org/)

A nom based parser, outputs a "Wavefront Obj" file which can be imported into blender and a Bevy app for visualization

## How to use

Parses `StdIn` as a gcode file - the SVG file is send to `StdOut` :-

```bash
cargo run --release -- < ../assets/bency.gcode > benchy.obj
```

see [todo](TODO.md)

## Future work

* As the gcode-nom library developes we could handle binary-gcode files.

* I have only tested against gcode files that use absolute positioning.
I must test will relative positioning.
