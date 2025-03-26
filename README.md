# gcode-nom

Rust 2021 Edition.

<div align="center">

<a href="https://crates.io/crates/gcode-nom"><img alt="crates.io" src="https://img.shields.io/crates/v/gcode-nom.svg"/></a>
<a href="https://docs.rs/gcode-nom/latest/gcode_nom" rel="nofollow noopener noreferrer"><img src="https://img.shields.io/crates/d/gcode_nom.svg" alt="Documentation"></a>

</div>

A library containing a full set of [nom](https://crates.io/crates/nom) parsers for decoding gcode files.

Both .gcode files and binary .bgcode files are recognized.

Based on this library the workspace contains a series of visualization tools

* gcode2obj - Generates "WaveFront Obj" files.
* gcode2svg - Generates SVG files.
* gcodeExtractThumbs - Extracts the images embedded in a binary-gcode file.
* bgcodeViewer - Generates a report by turning on all the logging and them attempts to parse the file.

I intend the parsers to be as strictly compliant as possible. This is under-going rapid development. Please create issues here, or send me gcode files which expose unimplemented sections.

See [nom](https://crates.io/crates/nom) - "A byte-oriented, zero-copy, parser combinator library"

## Tools

### gcode2obj

Generates obj files.

Pass the gcode file into StdIn - the obj file is sent to StdOut :-

```bash
cd gcode2obj
cargo run --release -- < ./assets/benchy.gcode > benchy.obj
```

If a filename is passed in as a command line parameter then a obj file will be dumped to StdOut.

```bash
cd gocode2obj
cargo run --release -- ../assets/X\ 6x6_0.15mm_PLA_MK3S_1h55m.gcode > X.obj
```

Which for example can be imported into blender for visualization.

![Benchy in Blender](https://github.com/martinfrances107/gcode-nom/blob/main/images/BlenderBenchy.png?raw=true)
![Lego bricks](https://github.com/martinfrances107/gcode-nom/blob/main/images/lego.png?raw=true)
Within blender :-

1) This obj has been "Imported".
2) Converted into a "Curve".
3) Finally a circular bevel object has been applied to make the object solid [ A circle to represent a 0.1mm fibre].

## gcode2svg

Generate svg files

![Benchy in Blender](https://raw.githubusercontent.com/martinfrances107/gcode-nom/367a7add7ed0dcad84ea20d21fd2076b559188b9/images/benchy.svg)

### How to use

Pass the gcode file into StdIn - the SVG file is sent to StdOut :-

```bash
cd gcode2svg
cargo run --release -- < ./assets/benchy.gcode > benchy.svg
```

If a filename is passed in as a command line parameter then a svg file will be dumped to StdOut.

```bash
cd gocode2svg
cargo run --release -- ../assets/X\ 6x6_0.15mm_PLA_MK3S_1h55m.gcode > X.svg
```

### gcodeExtractThumbs

 Iterates over all the embedded thumbnail block and save the images to disk

 ```bash
 cargo run --release  --  ../assets/both\ parts.bgcode
 ```

### bgcodeViewer

 Strict checking of binaries. validates blocks checksums, ensures 'block' parameters values are within valid ranges.

Pass the gcode file into StdIn - A summary file is written to StdOut

 ```rust
 cd bgcodeViewer
 cargo run --release  < ../assets/both\ parts.bgcode > summary.txt
 ```

## Future work

see [TODO](TODO.md)

A Bevy app?
