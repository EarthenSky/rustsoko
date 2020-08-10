# rustsoko
This is a sokoban solver which uses IDA* to solve puzzles with optimal pushes and best moves, as described here: http://www.sokobano.de/wiki/index.php?title=Level_format#Level_collection

Download XSokoban test suite from https://www.cs.cornell.edu/andru/xsokoban/download.html (maps included with binary as screen.*), then place them in the `/puzzles/` directory. Alternatively, one can choose a mapset from http://sokobano.de/de/levels.php to test.

Used the following instructions to cross-compile to macos:
- https://wapl.es/rust/2019/02/17/rust-cross-compile-linux-to-macos.html