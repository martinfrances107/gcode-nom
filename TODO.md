# TASKS

[] - Command::G0 non extruding moves have todo!() when it come to obj generation.

[] - BUG gcode2svg gears.svg has stringy parts.
    Something is not decoding correctly.

[] - BUG Missing compression algorithms
    HeadShrink (two modes to support)
    - Have a working implementation in gcode block but its no uniformly applied.

----

[] - Performacne
    producing gears.svg takes 1m20.128s!!!!

    I think read a whole file into memory might be a problem
    large files seem to blow up more than one would expect from just the size increase.
    In lots of places as I extract I clone into a string .. is that the problem

    crate a profile_target

    Benchmarks use [samply](https://crates.io/crates/samply). Maybe criterion.

[] - Identify any M-code that should not be dropped.

[] -Tests
     -- unit test for all Blocks.
     -- no unit testing of Block types.
     -- How to test binary blocks?

After basic functionality is complete.

[] - SVG Via cli rotate on Z axis and scale.
     how to merge sx, sy, sz into a single value?
