# rustsoko
This is a sokoban solver I made for my AI class, CMPT310, which uses IDA* and A* to solve puzzles with optimal pushes and best moves, as described [here](http://www.sokobano.de/wiki/index.php?title=Level_format#Level_collection). Also includes a random puzzle generator. 

### Features:
- Finds solutions to sokoban puzzles with optimal pushes and best moves
- Determines unsolvable solutions quickly with 'deadlock-hashing'
- Can generate sets of rectangular puzzles with set amounts of randomly distributed goals and wall

### Method:
- Rustsoko uses IDA* search on the pushes of a Sokoban puzzle, assuming that perfect moves were made, then collects all goal states of the optimal push length. 
- It then uses A* search to determine the perfect moves to connect the states in all of the collected solutions paths. 
- Finally, Rustsoko chooses the state with the smallest path length. Since all heuristics used are admissible, the solutions Rustsoko produces are push optimal with best moves.
- The main aspect push-optimality sacrifices is performance, because all goal nodes must be collected even thought the last time IDA* is run is the longest. 

### Notes:
- There are lots of puzzle sets here http://sokobano.de/de/levels.php
- The easier sets which this solver can handle most of are: 
  - Microban
  - SokEvo
  
  
### TODO:
- Memoize A* & flood fill calls.
- Create greedy A* & normal A* based (memoized) heuristics. -> due to memoization, greedy A* should perform even better than with manhattan distance.
