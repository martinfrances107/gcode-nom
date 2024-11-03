# gcodeToSvg

Rust 2021 Edition.

A G-code visualization tool written in [rust](https://www.rust-lang.org/)

A nom based parser, outputs a svg curve which can be imported into blender and a Bevy app for visuallization

## How to use

parses StdIn as a gcode file - the SVG file is send to StdOut :-

```bash
cargo run --release -- < ./assets/bency.gcode > bench.svg
```

This is undergoing rapid development.

see [todo](TODO.md)

## Future work

Maybe output a obj file?
