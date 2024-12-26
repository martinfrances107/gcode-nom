# gcodeToObj

Rust 2021 Edition.

A G-code visualization tool written in [rust](https://www.rust-lang.org/)

A nom based parser, outputs a "Wavefront Obj" file which can be imported into blender and a Bevy app for visualization

## How to use

Pass the gcode file in as 'StdIn' and the program will send the obj file to 'StdOut' :-

```bash
cargo run --release -- < ../assets/bency.gcode > benchy.obj
```

Which for example can be imported into blender for visualization.

![Benchy in Blender](images/BlenderBenchy.png)
![Lego bricks](images/lego.png)
Within blender :-

1) This obj has been "Imported".
2) Converted into a "Curve".
3) Finally a circular bevel object has been applied to make the object solid [ A circle to represent a 0.1mm fibre].

see [todo](TODO.md)

## Future work

* As the gcode-nom library developes we could handle binary-gcode files.

* I have only tested against gcode files that use absolute positioning.
I must test will relative positioning.
