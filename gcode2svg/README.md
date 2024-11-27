# gcodeToSvg

Rust 2021 Edition.

A G-code visualization tool written in [rust](https://www.rust-lang.org/)

A nom based parser, outputs a svg curve which can be imported into blender and a Bevy app for visualization

## How to use

Parses `StdIn` as a gcode file - the SVG file is send to `StdOut` :-

```bash
cargo run --release -- < ../assets/bency.gcode > benchy.svg
```

## Future work

Use the clap module to define option scale and rotation parameters.
