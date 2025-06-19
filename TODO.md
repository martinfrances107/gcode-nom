# TASKS

[] - BugFix: Handle M486 parsing associated with object naming is broken.
  /// M486 S3 A"cube copy 3" ; Indicate that the 4th object is starting now and name it

[] - parse g2/g3 Tests are not complete.

[] - G2, G3 - Clockwise Arcs/ CounterClockwise Arcs.

  (I,J) and R are mutually exclusive.
  Must reject case where R and I,J are not mutually exclusive.
  Uniformly unwind a misconception
  If X is specified and Y is omitted.
  The Y values becomes the current Y etc.
  This is not currently the case.

[] - G5 - Bezier curve paths

[] - Make a streaming parser.

  what speedup is expected 1.5s down to 500ms?

[] - cube -- add groups to .obj file
    -- produce render.

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
