# Notes on syntax design of regolith

A single file defines exactly one song. Songs can be composed of any
number of tracks (within reason), and any number of "sections" (I may
rename this to "movements"). The structure of a composition file
is more or less this:

```txt
[file start]
[file preamble]
[section one]
  [section one preamble]
  [track A]
    [measure 1]
    [measure 2]
    [measure 3]
    [repeat block]
      [measure 4]
      [measure 5]
      [measure 6]
    [end repeat block]
    [measure 7]
    [measure ...]
  [track B]
    [measure 1]
    [measure 2]
    [measure ...]
  [track ...]
[section two]
  [section two preamble]
  [track A]
    [measure ...]
  [track ...]
```

this is intended to be very structurally similar to classical music notation:

```txt       A                                         B
             1     2     3     4     5     6     7     8     9     10    11
TRUMPET      | --- | --- | --- | --- | --- |: -- | --- | -- :| --- | --- ||
PIANO    4/4 | --- | --- | --- | --- | --- |: -- | --- | -- :| --- | --- ||
DRUMS        | --- | --- | --- | --- | --- |: -- | --- | -- :| --- | --- ||
```
_A facsimile music sheet with 10 measures, two "sections", and three tracks._

"Preambles" contain information such as tempo, dynamics, time signature, and
key signature. Premables are relegated to the beginning of sections, or to
the top of the file itself; using the former option would allow the author to
set these parameters for individual sections, the latter for the song as
a whole.

Note: you should be able to swap around in tracks at any time; for example:

```regolith
---SECTION---
[1] | . . _ . | . . _ . | . . _ . |
[2] | _ . _ . | _ . _ . | _ . _ . |
[3] | . _ . _ | . _ . _ | . _ . _ |
```
