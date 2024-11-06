# gcode-nom

Rust 2021 Edition.

Contains two parsers:

* A streaming nom parser of "binary gcode files" ['.bgcode' extension].

* A parser for text based gcode files ['.gcode' extension].

I intend the binary gcode parser to be as stirictly complient as possible. This is under-going rapid development.

Based on this library, there are a series of visualisation tools within the workspace.

* bgcodeViewer - Strict checking of binaries.
* gocde2obj - Generates "WaveFront Obj" files.
* gcode2svg - Generates SVG files.

## bgcodeViewer

 A strict validator -- Reports on the quality of the binary.

 ```rust
 cd bgcdoeViewer
 cargo run --release  < ../assets/both\ parts.bgcode
 ```

## gcode2obj

Generates obj files.

### How to use

Pass the gcode file into StdIn - the obj file is sent to StdOut :-

```bash
cd gcode2obj
cargo run --release -- < ./assets/bency.gcode > bench.svg
```

Which for example can be imported into blender for visualisation.

![Benchy in Blender](images/BlenderBenchy.png)

Within blender :-

1) This obj has been "Imported".
2) Converted into a "Curve".
3) Finally a circular bevel object has been applied to make the object solid [ A circle to represent a 0.1mm fibre].

## gcode2svg

A nom based parser, outputs a svg file.

### How to use

Pass the gcode file into StdIn - the SVG file is sent to StdOut :-

```bash
cd gcode2svg
cargo run --release -- < ./assets/benchy.gcode > benchy.svg
```

## Future work

see [TODO](TODO.md)

A Bevy app?
