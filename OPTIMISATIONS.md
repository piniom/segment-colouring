# Solver optimisations: what changed and what it bought

This file covers the work on the Rust solver (`rust/`), done to make **MQ=5, MS=12** fit in memory and finish in reasonable time.

## Overall effect

| | before | after |
|---|---|---|
| MQ=5, MS=10, peak RSS | 6.6 GB | 0.17–0.3 GB (**~25–40× less**) |
| MQ=5, MS=10, time to depth 20 | ~286 s | ~125–131 s (**~2.2× faster**) |
| MQ=5, MS=11, stored states | ~1.35·10⁹ | ~87·10⁶ (**~15× fewer**) |
| MQ=5, MS=11, one saturated depth (remote run) | 29.6 min | ~4.2–4.6 min (**~6.5× faster**) |
| MQ=5, MS=12, projected memory | 1.5–2 TB | ~23–70 GB |

How these were measured:
- Local numbers come from a 16-thread macOS machine, using the `bench` binary. Its builds have stats enabled, which makes them about 10% slower than plain builds.
- Each row in the per-step tables compares a step with the code just before it. The steps were measured at different times, so their ratios don't multiply into the totals above exactly.
- The search is nondeterministic, so small differences (±5%) are noise.
- Every strategy produced on MQ=4 passed `cxx/strategy_check`.

## 1. Bug fix: back reductions never hit the cache

**Problem:** `try_moved_limit_back` removed its second event using the parent's length (`self.len() - 1`), not the clone's. That left a stray end event in the bits past `len`. Normalisation compares raw bits, so the same child normalised to one key on lookup and another key on store. Every back-reduced child was searched again from scratch. That happened about 24.6M times for only about 33k distinct states (MQ=4, MS=8).

**Fix:** one line, `clone.remove_at_index(clone.len() - 1)`. There are also `debug_assert!`s that no bits are set past `len` (in `remove_at_index` and `normalize_inner`), and regression tests, including a sweep over all reachable states. A defensive mask in `normalize` was also tried; it had no measurable cost, but it was dropped in favour of the assertions.

| config | before | after |
|---|---|---|
| MQ=4, MS=8, wall time | 1.49–1.87 s | 0.57–0.60 s (**~3×**) |
| MQ=4, MS=8, CPU time | 21.5–26.6 s | 7.8–8.3 s |
| MQ=4, MS=9, wall time | 13.5–13.7 s | 5.8–5.9 s (**~2.3×**) |
| memory | unchanged | unchanged |

**Side effect:** wins now come about 2 depths later, and strategy files are 2–5× larger. Running with 1 thread and with 16 threads gave the same win depth, so this isn't caused by threads.

## 2. Don't store losing states at the size limit (now the default)

**Change:** states with `size ≥ MS` are evaluated from their two reductions and not stored, except when they win, because `print_strategy` needs those. This is safe: every cycle contains an insert, and each insert lowers `depth`. The old behaviour is still available with `--store-at-limit`.

**Why it helps:** about 91.5% of stored states were at the size limit (MQ=5, MS=8).

Before packing (`--store-at-limit` against the default):

| config | memory | time |
|---|---|---|
| MQ=4, MS=8 | 39.9 → 25.5 MB (426k → 141k states) | 0.56 → 0.47 s |
| MQ=4, MS=9 | 671 → 150 MB | about the same |
| MQ=5, MS=9 | 665 → 59 MB | about the same |
| MQ=5, MS=10 | 6.6 GB → 666 MB | 286 → 280 s |

- **For MQ=5:** about **14× fewer states** and **about 10× less memory**, at the same speed. The default stores roughly what `--store-at-limit` stores at MS−1.
- **Shallow re-searches:** about 10× fewer.
- **Cost:** larger strategy files. For MQ=4, MS=8 they're 3.9k–17.6k lines, against 1.2k–4k with `--store-at-limit`.

## 3. Packed map entries

**Change:** the map went from `DashMap<Key, SmallVec<[BarrieredKnowledge; 3]>>` (a 33-byte slot) to `DashMap<CompressedState<7>, PackedKnowledge>` (**a 12-byte slot plus 1 control byte**).
- A single entry packs into a `u32`: a 3-bit tag, a 5-bit front barrier and a 5-bit back barrier, and a 10-bit payload (the move, the reduction side, or the losing depth).
- A key with several entries stores them in a side `overflow` map. That's about 0.1–0.3% of keys for MQ=5, but 9–45% for MQ=4.
- Locks are always taken in the same order: map, then overflow.

| config | `--store-at-limit` | default | time (default) |
|---|---|---|---|
| MQ=4, MS=9 | 671 → 350 MB | 150 → 110 MB | 5.0 → 4.4 s |
| MQ=5, MS=9 | 665 → 272 MB | 59 → 31 MB | 17.7 → 15.8 s |
| MQ=4, MS=10 | 6.2 → 4.3 GB | 1.37 → 1.24 GB | (single run, won 1 depth later) |
| MQ=5, MS=10 | 6.6 → 4.1 GB | 666 → 277 MB | 280 → 263 s |

- **Memory:** 37–58% less at the same state counts.
- **Speed:** about 5–10% faster, because a lookup no longer allocates a `Vec`.
- **Steps 2 and 3 together:** MQ=5, MS=10 went from **6.6 GB to 277 MB (~24×)**.

## 4. Pre-sizing the map (`--capacity <n>`)

**Change:** the map can be created at its final size, so it never grows. Growing leaves the freed old tables behind in the allocator. The default, 0, keeps the old behaviour.

| config | peak RSS | time |
|---|---|---|
| MQ=4, MS=9 | 110 → 94 MB | same |
| MQ=5, MS=10 | **281 → 172 MB** | 261 → 255 s |

That's about 30 bytes per stored state, down from about 50. To use it, pass the state count from a previous run.

## 5. Faster normalisation and key compression

**Problem:** profiling with `samply` showed `normalize_inner` taking **52%** of CPU and key compression **14%**. The map itself took about 6%.

**Change:**
- Normalisation renames colours in one pass over a local `u128`, and builds the mirror image without a loop (`swap_bytes`, a nibble swap, a shift, and an XOR of the end bits).
- Compression collects the key's codes in one `u64` and stores it with `to_be_bytes`. The bit layout is identical, so keys didn't change.
- The old implementations are kept in the tests as references. On more than 10k reachable states and 20k random ones, the new code must match them exactly.

| config | time |
|---|---|
| MQ=4, MS=9 | 4.36 → 3.05 s |
| MQ=5, MS=9 | 16.4 → 9.6 s |
| MQ=5, MS=10 | 256 → **160 s** (**1.6–1.7×**) |

## 6. Skip the lookup for states at the size limit

**Problem:** after step 2, losing states at the limit are never stored, yet every visit still normalised, compressed and looked them up. Move lookups missed 94% of the time.

**Change:** at-limit states go straight to their reductions (`find_strategy_from`). Wins are still stored, so strategy printing is unaffected.

| config | time | lookups |
|---|---|---|
| MQ=4, MS=9 | 2.86 → 2.15 s | 125M → 76M |
| MQ=5, MS=9 | 9.90 → 7.67 s | 496M → 329M |
| MQ=5, MS=10 | 166 → **125 s** (**1.25–1.33×**) | 6.75G → 4.46G |

Expansions and colour branches are unchanged; only the wasted lookups are gone.

## 7. Bug fix: duplicate winning entries

**Problem:** after step 6, a winning state at the limit was evaluated again on every visit, and each time its win was appended to the key's entries again. Memory and substate counts then crept up with every depth, even after the state count stopped growing. In the remote MS=11 run that was 3.9 winning entries per winning state, against 1.3 before.

**Fix:** a win isn't stored when the key already has a win with an equally or more confined barrier (`adds_nothing_given`), because that win already applies wherever the new one would.

| | duplicating | fixed |
|---|---|---|
| MQ=5, MS=10: winning entries per winning state | 23.0 | **2.2** |
| MQ=5, MS=10: total entries (5.96M states) | 6.54M | 5.99M |
| MQ=5, MS=10: peak RSS | 293 MB | 274 MB |
| MQ=4, MS=9: peak RSS | 169 MB | **110 MB** |
| time | 129 s / 2.31 s | 131 s / 2.20 s |

## Tried and dropped

| idea | result | decision |
|---|---|---|
| Other hashers (SipHash with `u64` key hashing, ahash, Fx) | all within ±4%, ranking changed between configurations; Fx spread keys less evenly | kept the default |
| Sequential move search near the size limit (`--sequential-levels 0–4`) | at most 3–6% at 16 threads, no trend with the level, mostly noise; the ~16% "rayon" time in profiles is search code inlined into rayon frames | removed |
| Raising the depth by `k > 1` per iteration (`--depth-step k`, `--depth-step-from d`) | slower in every case, more memory; see below | removed |
| Masking bits past `len` in `normalize` | no measurable cost, but it only hides bugs | replaced with `debug_assert!` |

### Depth step (`--depth-step k`)

**Idea:** iterative deepening raises the depth by 1 per iteration, and every iteration re-searches nearly all losing states, so a step of `k` might reach the same depth in `k` times fewer sweeps. Depths below `--depth-step-from` still rose by 1, and the last iteration was always at `-d`.

**Result:** 5 runs per variant with `bench` (16 threads), mean values. All 80 winning strategies passed `strategy_check`; every stepped loss ended with the same result and stored-state count as k=1 (`--verify-loss` didn't exist yet). For MQ=5 the step started at depth 13, where the state count stopped growing.

| config | k=1 | k=2 | k=3 | k=4 |
|---|---|---|---|---|
| MQ=4, MS=8 (win): time | 0.26 s | 0.28 s | 0.32 s | 0.36 s |
| MQ=4, MS=9 (win): time | 2.0 s | 3.2 s | 2.6 s | 4.2 s |
| MQ=4, MS=10 (win): time | 27 s | 42 s | 41 s | 69 s |
| MQ=4, MS=10: win depth / peak RSS | 12.4 / 0.9 GB | 13 / 1.5 GB | 13.2 / 1.2 GB | 15 / 2.8 GB |
| MQ=4, MS=7, `-d 30` (lose): time | 0.12 s | 0.11 s | 0.12 s | 0.12 s |
| MQ=5, MS=9, `-d 24` (lose): time | 9.7 s | 11.0 s | 11.5 s | 12.1 s |
| MQ=5, MS=9, `-d 40` (lose): time | 20.0 s | 23.5 s | 24.7 s | 25.5 s |
| MQ=5, MS=9, `-d 40`: iterations | 38 | 25 | 20 | 18 |
| MQ=5, MS=9, `-d 40`: expansions | 15.2M | 17.3M | 18.4M | 18.9M |
| MQ=5, MS=9, `-d 40`: peak RSS | 32 MB | 44 MB | 51 MB | 52 MB |

**Why it loses:** with step 1, a state stored as `Losing{d−1}` is searched again only by a path that reaches it with depth exactly `d`. With step `k` it's stored as `Losing{d−k}`, and every path reaching it with any depth from `d−k+1` to `d` searches it again, within the same iteration. Once the state count stops growing, one step-2 iteration did about 2.3× the expansions of a step-1 iteration, which more than cancels having half as many iterations. Winning runs also overshoot the winning depth, and a deeper search costs more.

## Tools and infrastructure

- **Cleanup:** only the main solver is left. `simple_state/` was flattened into `src/`, and the old `linear_axis` solver, the extra binaries, the scripts and unused code were removed.
- **Statistics feature** (`--features stats`): per-thread counters with no shared atomics, so it's safe at 200 threads. It counts:
  - lookups by caller and outcome,
  - expansions, moves, colour branches, reductions and writes.
  - It also gives map statistics and `--stats-json` output.
  - Without the feature it costs nothing; with it, runs are about 10% slower.
- **`bench` binary:** repeated runs per label, an `iterations` row, a frozen binary per label, peak RSS from `/usr/bin/time`, `strategy_check` on every strategy, `--timeout` (kills the run's process group), and side-by-side summaries.
- **Memory columns per depth:** `rss` / `peak_rss` and `map_table`, also written to the JSON.
- **Timing column:** each depth row shows its finish time, with seconds.
- **Tests:**
  - regression tests for both bugs;
  - reference checks for the fast normalisation and compression;
  - pack/unpack round-trips and a test pinning the 12-byte slot;
  - concurrent overflow updates;
  - counter consistency (`stats_test`);
  - both storage modes produce a printable strategy.
- **`AGENTS.md`:** how the solver works, how to test and measure it, and this optimisation history in short form.

## Where the time goes now

Memory is no longer the limit; time is. Once the state count stops growing, around depth 13, every depth searches every stored state again:
- expansions per depth are about equal to the stored states;
- every move is refuted by the first colour tried.

The ideas below came up but haven't been tried:
- Reporting when a depth changes nothing.
- Retrograde analysis: work proportional to the new wins instead of a full sweep per depth.
