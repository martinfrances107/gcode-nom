# TASKS

[] - G2, G3 - Clockwise Arcs/ CounterClockwise Arcs.

   In the parsing of G2, G3 - R form and IJ Form
      // Must added parse checking
      (I,J) and R are mutually exclusive.

   Provide a config PARAM "MM_PER_ARC_SEGMENT"

   OBJ implement renderer
    Compute theta start, theta end
    break the arc into a series of arc segments
    based on MM_PER_ARC_SEGMENT
    if arc length is 100mm and MM_PER_ARC_SEGMENT is 10mm per arc_segment
    then 10 steps!!.
    for loop add line segments.
    update current x,y

  tests reject case where R and I,J are not mutually exclusive.

  There are demos of a complete line with 2/3 arc segments. that can be asset/snapshot

[] - Unwind a misconception
  If X is specified and Y is omitted.
  The Y values becomes the current Y etc.
  This is not currently the case.

[] - both is broken in the arc branch.

[] - G5 - Bezier curve paths

[] - Make a streaming parser.

  what speedup is expected 1.5s down to 500ms?

[] - cube -- add groups to .obj file
    -- produce render.

[] - BUG gcode2svg and gcode2obj "gears" has stringy parts.
    Something is not decoding correctly.

    when moving from gcode block to gcode block
    how does the head move .. Is the position from the last
    section retained.

    bench2-mk4a Loop at the tips of the ridges
     They are also stringy.

    "G0 and G1 Must be handled uniformly" maybe the fix.

[] - BUG Missing compression algorithms
     HeadShrink (two modes to support)
     Have a working implementation in gcode block but its no uniformly applied.

     HeatShrink is specific for GCODE::Commands ..
     Does this apply to all sub blocks?

[] - Image gallery choose visually appealing obj's and take a collage

----

[] - Performance

    "Recognizers usually take an input array, and return either
    an error, or a tuple containing the recognized part, and the
    rest of the input. This tuple is often a cause of a lot of data
    copying between parsers."

    <http://spw15.langsec.org/papers/couprie-nom.pdf>

    The repository mention in the pdf have been renamed

    <https://github.com/rust-bakery/parser_benchmarks>


    gear-holder-print-in-place_04n_022mm_pla_mk4_6h49m.bgcode

    is the target to test against.
    Need to add two transformers -- to convert to a streaming parser.

    crate a profile_target

    Benchmarks use [samply](https://crates.io/crates/samply). Maybe criterion.

    DHAT

[] - Identify any M-code that should not be dropped.

[] -Tests
     -- unit test for all Blocks.
     -- no unit testing of Block types.
     -- How to test binary blocks?

After basic functionality is complete.

[] - SVG Via cli rotate on Z axis and scale.
     how to merge sx, sy, sz into a single value?
