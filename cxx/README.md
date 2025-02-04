# Game

The Bounded On-Line Proper Interval Coloring Game is played in rounds by two players: Spoiler and Algorithm.

# Strategy files

First line of the strategy file contains two numbers: the maximum clique size, and the number of colors that Spoiler wants to force.
Each following line contains the description of a state and the description of a move by Spoiler.
Description of a state is a string in which:

 * A capital letter $x$ in `A-Z` denotes the left endpoint of an interval colored `lower($x$)`.
 * A letter $x$ in `a-z` dentoes the right endpoint of an interval colored $x$.
 * A character `[` denotes the left barrier.
 * A character `]` denotes the right barrier.

Description of a move is either:

 * A character `<` means that Spoiler pushes the right barrier minimally to the left so that one interval is dropped.
 * A character `>` means that Spoiler pushes the left barrier minimally to the right so that one interval is dropped.
 * Two numbers $x$ $y$ mean that Spoiler introduces new interval with endpoints $x$ and $y$.

## Example

File `2_2.strategy` contains an example Spoiler strategy, that forces Algorithm to use 3 colors when playing with maximum clique size 2.
There are 6 states in the strategy.

# `strategy_check` application

`strategy_check` reads strategy files (provided as command line arguments) and checks that they provide a winning strategy for Spoiler, i.e.:

 * Every described move is valid.
 * Every described state leads, by a sequence of moves, to Spoiler win.
