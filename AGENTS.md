# AGENTS.md

Search for Spoiler (Builder) strategies in the on-line proper interval colouring game:
force any colouring algorithm to use `2·MQ − 1` colours while never creating a clique larger than `MQ`.
Current goal: **MQ = 5** (with max state size `MS = 12`).

## Repository layout

- `rust/`: the solver. All active work happens here.
  - `src/simple_state/`: **the current solver**, run through the `bm_find_strategy` binary.
  - `src/linear_axis/` + `src/main.rs`: older solver (`default-run` binary `segment-colouring`). It isn't developed any more.
  - `results.txt`: logs of earlier long runs (per-depth timings and state counts).
  - `strategy.txt`: **tracked in git**, and the default output path of `bm_find_strategy`. A run overwrites it.
- `cxx/`: C++ tools: `strategy_find`, `strategy_check`, `strategy_draw`. `README.md` there defines the game and the strategy file format.

## Running

```bash
cd rust
cargo run --release --bin bm_find_strategy -- -q <MQ> -s <MS> [-d <max depth, default 255>] [-o <file, default strategy.txt>]
```

The program prints one row per iterative-deepening depth: time, result, and counts of winning and total states and substates. On a win it writes the strategy file.

## Environment

- Production runs happen on **external machines with up to ~200 threads and 500 GB – 2 TB RAM**. Memory is the limiting resource for MQ=5.
- The search is **multithreaded** (rayon + `DashMap`). Every change must hold up under heavy parallelism:
  - avoid shared atomics or locks in the hot path,
  - keep in mind that the shared state map, including `InProgress` entries, is seen by every thread.
- Parallel search is **nondeterministic**: colour order is shuffled and the first winning move found wins. State counts and timings vary between runs, so compare several runs, not one.

## Testing changes (important)

1. Run `cargo test --release` in `rust/`.
2. Run the solver on **MQ=4, MS=8**, or bigger if needed:
   `cargo run --release --bin bm_find_strategy -- -q 4 -s 8 -o <file>`. This wins in a few seconds.
3. **Validate the produced strategy with the C++ checker. Don't check strategy output by hand:**
   ```bash
   cd cxx && make strategy_check   # first time only
   ./strategy_check ../rust/<file_to_check>
   ```
   It must print `Strategy is correct.`
4. **Never test on MQ=5.** It takes hours.

## How the current solver works (`rust/src/simple_state`)

### State (`state/mod.rs`, `state/limits.rs`)
- `State<MAX_CLIQUE>` is a single `u128`:
  - up to 28 events × 4 bits, then `len`, `limit_front` and `limit_back` (5 bits each).
  - An event is `colour` in bits 0–2 and an end flag in bit 3. So there are at most 8 colours.
- Text form (`state/string.rs`): `A–H` = start of colour 0–7, `a–h` = end. `[` and `]` are the limits.
  Events outside the limits belong to segments that can no longer be touched.
- Segments are proper intervals (none contains another), so end events come in the same order as start events.
- `size()` = number of segments = `len / 2`.
- **Normalisation** (`normalize`):
  - renames colours in order of first appearance,
  - then compares with the mirrored state (`flip`) and keeps the smaller one by event bits.
  - It returns whether a flip happened. Moves and barriers stored for a state are flipped to match.

### Moves (`simple_state/mod.rs`)
- **Insert** a segment `Move(start, end)` inside the limits, keeping it proper and the clique ≤ MQ (`valid_segment_ends`).
- The Algorithm answers with any allowed colour from `0..min(2·MQ−2, colours_used+1)`.
- If no colour is allowed, the move wins vacuously: the Algorithm would need colour `2·MQ−1`.
- **Reductions** (`try_moved_limit_front` / `try_moved_limit_back`): drop the first or last segment and move the limit past it.

### Search (`find/mod.rs`, `find/root.rs`)
- `find_strategy_root` runs iterative deepening over depth `3..=D` and keeps the map between iterations.
- `find_strategy` normalises the state, then `find_strategy_inner`:
  1. **Look up** the map. `Winning` → win. `Losing{d}` with `d ≥ depth` → lose. `InProgress` → lose (cuts cycles and work other threads are already doing).
  2. `depth == 0` → store `Losing{0}`.
  3. Store `InProgress`.
  4. **Try reductions.** Below max size they're tried at depth 1. At `size ≥ MS` they're tried at full depth, because they're the only option there. **Reductions don't decrease depth; inserts do.**
  5. At `size ≥ MS` → store `Losing{depth}`.
  6. Otherwise try all insert moves in parallel (`par_iter().find_map_any`). A move wins if **every** allowed colour leads to a win (`StateWithMove::find_strategy`, colours shuffled, sequential).
  7. Store `Winning(move)` or `Losing{depth}`.

### State map (`find/search_state.rs`, `state/compressed.rs`, `state/find_barrier.rs`)
- The map is `DashMap<CompressedState<7>, RepresentativeKnowledge>` with `2^15` shards.
- **Key** (`CompressedState<7>`): events only, without the limits.
  - A start takes 4 bits (`0ccc`), an end takes 1 bit (`1`). End colours follow from start order.
  - The first event, start of colour 0, is implicit.
  - Total `5·MS − 4` bits, so 56 bits for MS=12.
- **Value** (`RepresentativeKnowledge`): a `SmallVec<[BarrieredKnowledge; 3]>`.
  - A `BarrieredKnowledge` is a `FindBarrier` plus a `FindStatus`.
  - `FindBarrier` = `(limit_front, len − limit_back)`: the number of events outside each limit. More outside means more confined.
  - A `Winning` entry applies to states at most as confined as itself. `Losing` / `InProgress` entries apply to states at least as confined.
  - `update_status` drops entries that the new one makes redundant. For symmetric states it also stores the mirrored entry.
- `FindStatus` is one of: `Winning(WinningMove)` (an insert move or a reduction), `Losing { depth }`, `InProgress`, `Unknown`.

### Strategy output (`print_strategy/mod.rs`)
- `print_strategy` walks the stored `Winning` moves from the empty state and writes the cxx strategy format:
  - first line: `MQ 2·MQ−1`,
  - then one line per state: `<state> <move>`, where the move is `l r`, `>` or `<`.
- It relies on every state the strategy reaches having a stored `Winning` entry.

## Current scale (for reference)

- Per entry: 7-byte key + 24-byte value = 32 bytes, plus hashbrown's control byte and load-factor slack, so roughly 38–75 bytes per state.
- MQ=5, MS=11 (`results.txt`):
  - about 1.35·10⁹ states, about 98.5% of them losing;
  - the state count stops growing after depth ≈ 13;
  - no win found up to depth 27.
- The large majority of stored states are at `size == MS`. For MQ=5, MS=8 that's 91.5%.
