# gcode-nom

Rust 2021 Edition.

Parses strings into G-code comamnds.

This allows a series of G-code visualization tool.

```bash
gcode2svg
gocde2obj
```

## gcode2svg

A nom based parser, outputs a svg file

### How to use

parses StdIn as a gcode file - the SVG file is send to StdOut :-

```bash
cargo run --release -- < ./assets/bency.gcode > bench.svg
```

This is undergoing rapid development.

see [TODO](TODO.md)

## Future work

### gcode2obj

which can be imported into blender and a [Bevy](<https://bevyengine.org/>) app for visuallization
