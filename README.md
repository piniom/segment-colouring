# Segment colouring
Simulates an on-line proper segment colouring game between a builder and a colouring algorithm. 
Try to force ANY colouring algorithm to use a desired number of colours without exceeding the max clicque size.

## Run the simulation

Install [Rust](https://www.rust-lang.org/tools/install).

### Run the simulation
```bash
cargo run --release -- 5 3 7
```
Description of the arguments:
```bash
cargo run -- --help
```
The `--release` flag is important for simulation speed.

<!-- depth = 9:
[
    0,
    1,
    6560,
    346640,
    2330184,
    4322212,
    2986207,
    866257,
    105179,
    4334,
] 

col = 8, depth = 3
Total: 105179
Succ : 895
Fail : 104284
-->