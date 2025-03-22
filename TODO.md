# TASKS

[] - cube -- add groups to .obj file
    -- produce render.

[] - BUG gcode2svg and gcode2obj "gears" has stringy parts.
    Something is not decoding correctly.

    when moving from gcode block to gcode block
    how does the head move .. Is the position from the last
    section retained.

[] - BUG Missing compression algorithms
    HeadShrink (two modes to support)
    Have a working implementation in gcode block but its no uniformly applied.

    HeatShrink is specific for GCODE::Commands ..
    Does this apply to all sub blocks?

[]  - Image gallery choose visually appealing obj's and take a collage

----

[] - Performance

    extract thumbs has performance issues.

    gear-holder-print-in-place_04n_022mm_pla_mk4_6h49m.bgcode

    HashMap - create a feature

    Hashmap shows
    I think read a whole file into memory might be a problem
    large files seem to blow up more than one would expect from just the size increase.
    In lots of places as I extract I clone into a string .. is that the problem

    crate a profile_target

    Benchmarks use [samply](https://crates.io/crates/samply). Maybe criterion.

    DHAT, flamegraph?

[] - use rustc-hash with config options to fall back to std::hash
    if the library takes gocde files pull from a public network
    the it may be subject to DOS!!!

[] - Identify any M-code that should not be dropped.

[] -Tests
     -- unit test for all Blocks.
     -- no unit testing of Block types.
     -- How to test binary blocks?

After basic functionality is complete.

[] - SVG Via cli rotate on Z axis and scale.
     how to merge sx, sy, sz into a single value?
