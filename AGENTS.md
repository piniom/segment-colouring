# AGENTS.md

Search for Spoiler (Builder) strategies in the on-line proper interval colouring game:
force any colouring algorithm to use `2·MQ − 1` colours while never creating a clique larger than `MQ`.
Current goal: **MQ = 5** (with max state size `MS = 12`).

## Repository layout

- `rust/`: the solver. All active work happens here.
  - `src/main.rs`: the solver binary (`segment-colouring`, the `default-run`), with CLI arguments via clap.
  - `src/bin/bench.rs`: the `bench` binary, for benchmarking and comparing variants (see Measuring).
  - `src/{state,find,print_strategy}/` and `src/moves.rs`: the solver modules.
  - The crate is binary-only, and the compiler flags unused code; remove it rather than keep it.
  - The older `linear_axis` solver and other binaries were removed; they're still in git history.
  - `results.txt`: logs of earlier long runs (per-depth timings and state counts).
  - `strategy.txt`: **tracked in git**, and the default output path of the solver. A run overwrites it.
- `cxx/`: C++ tools: `strategy_find`, `strategy_check`, `strategy_draw`. `README.md` there defines the game and the strategy file format.

## Running

```bash
cd rust
cargo run --release -- -q <MQ> -s <MS> [-d <max depth, default 255>] [-o <file, default strategy.txt>]
```

The program prints one row per iterative-deepening depth: time, result, counts of winning and total states and substates, and memory. On a win it writes the strategy file.

- **Memory columns** ([src/memory.rs](rust/src/memory.rs)):
  - `rss` (Linux, from `/proc/self/status`) or `peak_rss` (macOS, from `getrusage`), read when each depth ends;
  - `map_table`: the hash table's own size (capacity × 13 bytes). The difference between the two is leftover tables from growing (avoid it with `--capacity`), the overflow map, and thread stacks.
- **JSON:** `--stats-json` writes the same values in each `depth` record under `memory`.

## Environment

- Production runs happen on **external machines with up to ~200 threads and 500 GB – 2 TB RAM**. Memory is the limiting resource for MQ=5.
- The search is **multithreaded** (rayon + `DashMap`). Every change must hold up under heavy parallelism:
  - avoid shared atomics or locks in the hot path,
  - keep in mind that the shared state map, including `InProgress` entries, is seen by every thread.
- Parallel search is **nondeterministic**: colour order is shuffled and the first winning move found wins. State counts and timings vary between runs, so compare several runs, not one.

## Testing changes (important)

1. Run the tests in `rust/`:
   - `cargo test --release` is fast (about 15 s).
   - `cargo test` also checks the `debug_assert!` invariants, but it's slow (about 2.5 min).
   - Add `--features stats` to either command to also run the counter-consistency test, `find::stats_test`. It also fails when reduction lookups miss the map.
2. Run the solver on **MQ=4, MS=8**, or bigger if needed:
   `cargo run --release -- -q 4 -s 8 -o <file>`. This wins in a few seconds.
3. **Validate the produced strategy with the C++ checker. Don't check strategy output by hand:**
   ```bash
   cd cxx && make strategy_check   # first time only
   ./strategy_check ../rust/<file_to_check>
   ```
   It must print `Strategy is correct.`
4. **Never test on MQ=5.** It takes hours.

## Measuring (statistics feature)

- **Turning it on:** `cargo build --release --features stats` enables search counters (`src/stats.rs`).
  - Without the feature the counter calls compile to nothing; there's no measurable overhead.
  - With it, runs are about 10% slower.
  - Each thread increments its own counters, with no shared atomics, so it's safe at 200 threads.
- **What's counted** (per depth iteration, bucketed by state size):
  - map lookups by caller (root / insert move / front reduction / back reduction) and outcome (`win` / `lose_deep` / `lose_shallow` / `in_progress` / `unknown`),
  - expansions, generated moves, colour branches, reduction checks and wins, and map writes.
- **Map statistics** (collected only with the feature): entries per key and stored states by size.
- **Console:** a stats build prints one extra line per depth, including the unknown rate (cache misses) per caller.
- **JSON:** `--stats-json <file> [--label <name>]` writes one JSON object per line:
  - `run`: parameters, thread count, whether stats are enabled,
  - `depth`: one per iteration (result, time, map statistics, counters),
  - `end`: final result, total time, stored states.

  It works without the feature too, but then only the timing and basic map numbers are written.
- **Benchmarking variants:** use the `bench` binary ([src/bin/bench.rs](rust/src/bin/bench.rs)), from `rust/`.
  ```bash
  cargo run --release --bin bench -- run -n 5 -l before -- -q 4 -s 8   # build with stats, 5 runs, check each strategy
  # ...change the code...
  cargo run --release --bin bench -- run -n 5 -l after -- -q 4 -s 8    # summary compares both labels side by side
  cargo run --release --bin bench -- summary <files.jsonl>...           # table for existing statistics files
  ```
  - Options:
    - `-d <dir>` sets the output directory (default `bench-out/`),
    - `--no-stats` measures a build without counters,
    - `--no-build` reuses a label's binary,
    - the default label is the current commit (read-only git), plus `-dirty`.
  - Don't pass `-o`, `--stats-json` or `--label` in the solver arguments; `bench` sets them.
  - `bench run` freezes each label's binary in the output directory, records peak RSS (BSD/GNU `/usr/bin/time`), and runs `cxx/strategy_check` on every strategy.
  - **Reading the summary:**
    - a high `unknown %` for the front or back reductions means lookups miss the cache (a key bug);
    - `re-searched (shallow)` counts states searched again only because their cached result was too shallow.
- **Profiling:** use `samply`, built with debug info in a separate target directory:
  ```bash
  CARGO_TARGET_DIR=/tmp/target-prof CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release
  samply record --save-only --unstable-presymbolicate -o prof.json.gz -- /tmp/target-prof/release/segment-colouring -q 5 -s 9 -d 16 -o /dev/null
  samply load prof.json.gz   # opens the Firefox Profiler
  ```
  Most hot code is `#[inline(always)]`. It shows up inside `find_strategy_from` and `get_knowledge` unless you look at inlined frames.
- **Optimisation history (measured with `bench` on MQ=5, MS=9–10):**
  - Fast normalisation (single-pass colour renaming, branch-free mirror) plus word-at-a-time key compression: 1.6–1.7× faster. Before that, they took about 66% of CPU.
  - Other hashers made no difference (SipHash, ahash and Fx were all within ±4%).

## How the current solver works (`rust/src`)

### State (`state/mod.rs`, `state/limits.rs`)
- `State<MAX_CLIQUE>` is a single `u128`:
  - up to 28 events × 4 bits, then `len`, `limit_front` and `limit_back` (5 bits each).
  - An event is `colour` in bits 0–2 and an end flag in bit 3. So there are at most 8 colours.
- Text form (`state/string.rs`): `A–H` = start of colour 0–7, `a–h` = end. `[` and `]` are the limits.
  Events outside the limits belong to segments that can no longer be touched.
- Segments are proper intervals (none contains another), so end events come in the same order as start events.
- `size()` = number of segments = `len / 2`.
- **Invariant: event bits past `len` are zero.**
  - `normalize`, `is_symmetric`, `PartialEq` and `Hash` compare the raw bits, so stray bits silently break state-map lookups.
  - `debug_assert!`s in `remove_at_index` and `normalize_inner` enforce this.
- **Normalisation** (`normalize`):
  - renames colours in order of first appearance,
  - then compares with the mirrored state (`flip`) and keeps the smaller one by event bits.
  - It returns whether a flip happened. Moves and barriers stored for a state are flipped to match.

### Moves (`moves.rs`)
- **Insert** a segment `Move(start, end)` inside the limits, keeping it proper and the clique ≤ MQ (`valid_segment_ends`).
- The Algorithm answers with any allowed colour from `0..min(2·MQ−2, colours_used+1)`.
- If no colour is allowed, the move wins vacuously: the Algorithm would need colour `2·MQ−1`.
- **Reductions** (`try_moved_limit_front` / `try_moved_limit_back`): drop the first or last segment and move the limit past it.

### Search (`find/mod.rs`, `find/root.rs`)
- `find_strategy_root` runs iterative deepening over depth `3..=D` and keeps the map between iterations.
- `find_strategy` normalises the state, then `find_strategy_inner`:
  1. **Look up** the map. `Winning` → win. `Losing{d}` with `d ≥ depth` → lose. `InProgress` → lose (cuts cycles; measured hits come from the state's own ancestors, not other threads).
     - **By default** (`SearchState::store_at_limit = false`), a state at `size ≥ MS` stops here and is evaluated from its two reductions without being stored. Only its wins are stored, which `print_strategy` needs. That's safe because every cycle contains an insert, which lowers `depth`.
  2. `depth == 0` → store `Losing{0}`.
  3. Store `InProgress`.
  4. **Try reductions.** Below max size they're tried at depth 1. At `size ≥ MS` they're tried at full depth, because they're the only option there. **Reductions don't decrease depth; inserts do.**
  5. At `size ≥ MS` → store `Losing{depth}` (only reached with `--store-at-limit`).
  6. Otherwise try all insert moves in parallel (`par_iter().find_map_any`). A move wins if **every** allowed colour leads to a win (`StateWithMove::find_strategy`, colours shuffled, sequential).
  7. Store `Winning(move)` or `Losing{depth}`.

### State map (`find/search_state.rs`, `state/compressed.rs`, `state/find_barrier.rs`)
- The map is `DashMap<CompressedState<7>, PackedKnowledge>` with `2^15` shards, plus a side map `overflow`.
- **Key** (`CompressedState<7>`): events only, without the limits.
  - A start takes 4 bits (`0ccc`), an end takes 1 bit (`1`). End colours follow from start order.
  - The first event, start of colour 0, is implicit.
  - Total `5·MS − 4` bits, so 56 bits for MS=12.
- **Value:** each key holds one or more `BarrieredKnowledge` entries.
  - A `BarrieredKnowledge` is a `FindBarrier` plus a `FindStatus`.
  - `FindBarrier` = `(limit_front, len − limit_back)`: the number of events outside each limit. More outside means more confined.
  - A `Winning` entry applies to states at most as confined as itself. `Losing` / `InProgress` entries apply to states at least as confined.
  - `update_status` drops entries that the new one makes redundant. For symmetric states it also stores the mirrored entry.
- **Packing** (`PackedKnowledge`):
  - A single entry is packed into the map's `u32` value: a 3-bit tag, 5-bit barrier front and back, and a 10-bit payload (the move, the reduction side, or the losing depth). Event indexes are ≤ 28, so 5 bits suffice. A slot is 12 bytes, pinned by a test.
  - Keys with several entries store them all in `overflow`, and their map value is the `OVERFLOW` marker. That's about 0.3% of keys for MQ=5, but 9–45% for MQ=4.
  - **Locking:** writers hold the key's map entry while they change `overflow`, and readers hold their map guard while they read it. The order is always map, then overflow.
- `FindStatus` is one of: `Winning(WinningMove)` (an insert move or a reduction), `Losing { depth }`, `InProgress`, `Unknown`.

### Strategy output (`print_strategy/mod.rs`)
- `print_strategy` walks the stored `Winning` moves from the empty state and writes the cxx strategy format:
  - first line: `MQ 2·MQ−1`,
  - then one line per state: `<state> <move>`, where the move is `l r`, `>` or `<`.
- It relies on every state the strategy reaches having a stored `Winning` entry.

## Current scale (for reference)

- **Per slot:** 7-byte key + 4-byte packed value = 12 bytes, plus hashbrown's control byte and load-factor slack.
  - **`--capacity <n>`** pre-sizes the map, so it never grows (growing leaves freed tables behind). At MQ=5, MS=10 this cut peak RSS from 281 to 172 MB (about 30 bytes per state). Pass the state count you expect, for example from a previous run.
  - Measured peak RSS is about 50 bytes per stored state (MQ=5, MS=10).
  - Before packing it was about 76 bytes.
- **Not storing at the limit** (the default) stores about 14× fewer states for MQ=5 than `--store-at-limit`, at the same speed. It stores roughly what `--store-at-limit` stores for MS−1. Its cost is larger strategy files (3.9k–17.6k lines against 1.2k–4k for MQ=4, MS=8).
- **Projection for MQ=5, MS=12** (extrapolated from small sizes plus the MS=11 count below): about 2·10¹⁰ states with `--store-at-limit`, against about 1.35·10⁹ by default. With packing, that's roughly 70 GB.
- MQ=5, MS=11 (`results.txt`):
  - about 1.35·10⁹ states, about 98.5% of them losing;
  - the state count stops growing after depth ≈ 13;
  - no win found up to depth 27.
- The large majority of stored states are at `size == MS`. For MQ=5, MS=8 that's 91.5%.
