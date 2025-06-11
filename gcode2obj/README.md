# gcodeToObj

Rust 2021 Edition.

<div align="center">

<a href="https://crates.io/crates/gcode2obj"><img alt="crates.io" src="https://img.shields.io/crates/v/gcode2obj.svg"/></a>
<a href="https://docs.rs/gcode2obj/latest/gcode2obj" rel="nofollow noopener noreferrer"><img src="https://img.shields.io/crates/d/gcode2obj.svg" alt="Documentation"></a>

</div>

A G-code visualization tool written in [rust](https://www.rust-lang.org/)

A nom based parser, outputs a "Wavefront Obj" file which can be imported into blender for visualization.

Both .gcode files and binary .bgcode files are accepted.

## Performance

Currently 9.9MByte bgcode file can be processed into a 16MBytes obj file in 1.5secs.

[ See the git repository associated with this project...
assets/gear-holder-print-in-place_04n_022mm_pla_mk4_6h49m.bgcode ]

## How to use

Pass the gcode file in as 'StdIn' and the program will send the obj file to 'StdOut' :-

```bash
cargo run --release -- < ../assets/benchy.gcode > benchy.obj
```

Binary ".bgcode" files must be passed in by filename :-

```bash
## Binary bgcoode files must be passed in as an argument because they are not utf-8 encoded
cargo run --release --  ../assets/both\ parts.bgcode > both.obj
```

Here the obj files are imported in to blender and processed.

![Benchy in Blender](<https://github.com/martinfrances107/gcode-nom/blob/main/images/BlenderBenchy.png?raw=true>)
![Lego bricks](https://github.com/martinfrances107/gcode-nom/blob/main/images/lego.png?raw=true)
![gears](https://github.com/martinfrances107/gcode-nom/blob/main/images/gears.png?raw=true)

Within blender :-

1) This obj has been "Imported".
2) Converted into a "Curve".
3) Finally a circular bevel object has been applied to make the object solid [ A circle to represent a 0.1mm fibre].

## Future work

* Make this nom-parser a "streaming / zero copy" parser. So only a small fragment of the large files is memory.

* I have only tested against gcode files that use absolute positioning.

* I must test with code than uses relative positioning.

* Convert "G5 - Bézier Cubic Spline" commands into the equivalent "obj" spline by defining a basis     matrix.  Support is not universal I could fall back to a series of line segments.