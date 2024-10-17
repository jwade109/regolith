# AST testing

Hello!

```regolith
CMAJOR

4/4 .

------

F3

------

120BPM

[: ./2 | F2 ./3 | . . | :] [2] . |

---SECTION3---

60BPM F2 . . .

```

plaintext embedded in source code is baller

```regolith
------
CMAJOR 120BPM F3 . ./2 G3 .:3
```

The abstract syntax tree for this file is

```txt
[top]
   [section] <implicit-section>
      [preamble]
         [scale] CMAJOR
         [endline]
         [ts] 4/4
      [staff]
         [note] .
         [endline]
   [section] ------
      [preamble]
         [endline]
      [staff]
         [pitch] F3
         [endline]
   [section] ------
      [preamble]
         [endline]
         [tempo] 120BPM
         [endline]
      [staff]
         [repeat] x1
            [note] ./2
            [mb] |
            [pitch] F2
            [note] ./3
            [mb] |
            [note] .
            [note] .
            [mb] |
         [track] [2]
         [note] .
         [mb] |
         [endline]
   [section] ---SECTION3---
      [preamble]
         [endline]
         [tempo] 60BPM
      [staff]
         [pitch] F2
         [note] .
         [note] .
         [note] .
         [endline]
   [section] ------
      [preamble]
         [endline]
         [scale] CMAJOR
         [tempo] 120BPM
      [staff]
         [pitch] F3
         [note] .
         [note] ./2
         [pitch] G3
         [note] .:3
         [endline]
[end]
```
