# rustsoko
This is a sokoban solver I made for my AI class, CMPT310, which uses IDA* to solve puzzles with optimal pushes and best moves, as described [here](http://www.sokobano.de/wiki/index.php?title=Level_format#Level_collection). Also includes a random puzzle generator. 

### Features:
- Finds solutions to sokoban puzzles with optimal pushes and best moves
- Determines unsolvable solutions quickly with 'deadlock-hashing'
- Can generate sets of rectangular puzzles with set amounts of randomly distributed goals and wall

### Notes:
- There are lots of puzzle sets here http://sokobano.de/de/levels.php
- The easier sets which this solver can handle most of are: 
  - Microban
  - SokEvo
