# Segment colouring
Simulates an on-line proper segment colouring game between a builder and a colouring algorithm. 
Try to force ANY colouring algorithm to use a desired number of colours without exceeding the max clicque size.

## Run the simulation

Install [Rust](https://www.rust-lang.org/tools/install).

### Run the simulation
```bash
cargo run --release -- -q 4 -s 8   # max clique 4, max state size 8
```
Description of the arguments:
```bash
cargo run -- --help
```
The `--release` flag is important for simulation speed.
