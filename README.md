# gcode-nom

Rust 2021 Edition.

Parses strings into G-code comamnds.

This allows a series of G-code visualization tool.

```bash
gcode2svg
gocde2obj
```

## gcode2obj

Generates obj file

Which for example can be imported into blender for visualisation

![Benchy in Blender](images/BlenderBenchy.png)

Within blender :-

1) This obj has been "Imported".
2) Converted into a "Curve".
3) Finally  circular bevel object has been applied to make the object solid [ A circle to represent a 0.1mm fibre].

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

A Bevy app?
