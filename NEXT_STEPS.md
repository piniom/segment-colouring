# Next steps: experiments and solver improvements

This is a work brief for an implementer. Read `AGENTS.md` first: it describes the game, the solver, and how to test. This file says what to build next and why.

## Background

The solver searches for a Builder (Spoiler) strategy that forces `2·MQ − 1` colours with clique size at most `MQ`, on a board of at most `MS` segments.

What we know (all with the default colour target `2·MQ − 1`):

| MQ | Largest MS where Builder loses | Smallest MS where Builder wins | Depth of that win |
|---|---|---|---|
| 2 | 2 | 3 | 4 |
| 3 | 4 | 5 | 7 |
| 4 | 7 (winning states stop at 158 from depth 11) | 8 | 13 |
| 5 | 10 (winning states stopped growing at depth 18) | unknown | unknown |

- MQ=5, MS=11: new wins per depth fell to 51 at depth 25, so it is almost certainly a loss, but the run was not finished.
- MQ=5, MS=12: still running on the remote machine, about 2 hours per depth. Not decided, but it now looks like a loss:
  - new wins per depth peaked at depth 17 (212M), then 199M at depth 18 and 101M at depth 19;
  - MS=11 showed the same shape before it levelled off (peak at depth 16, then about ×0.6, ×0.4, ×0.3 per depth);
  - If it follows MS=11, new wins reach zero around depth 32.
  - The printed winning/total ratio (36% at depth 19) is misleading: the winning count includes states at the size limit, and the total does not include losing states at the limit, because those are not stored. See Task 0. The reliable number is `total − winning`, the non-winning states below the limit: it fell from 1.353·10⁹ at depth 13 to 1.313·10⁹ at depth 19, so about 3% of the states below the limit have turned winning. For MS=11 the same figure is about 1%, for MS=10 about 0%.

The open question is whether MQ=5 is the first case where Builder cannot force `2·MQ − 1` colours. The tasks below give us:

1. evidence about whether MQ=5 behaves differently (Task 1),
2. a losing search that ends by itself (Task 2),
3. faster runs (Tasks 3 and 4),
4. the ability to run MS=13 (Task 5),
5. a check of whether the search reports "lose" at depths where a win exists (Task 6),
6. a test, on MQ=2, 3, 4, of the lemma that a proof for unbounded boards needs (Task 7; it turned out to be vacuous there),
7. a checker for a certificate that the Algorithm survives on an unbounded board (Task 8). This is now the main line towards a proof for MQ=5.

## Rules for all tasks

- **No git writes.** The user manages git. Use only read-only git commands.
- **Never overwrite `rust/strategy.txt`.** Always pass `-o` with a scratch path.
- **Defaults must not change.** Without the new flags, the search must behave exactly as before.
- **MQ=5 runs are allowed only where a task says so**, and always with the time cap given there. macOS has no `timeout` command; use `perl -e 'alarm shift; exec @ARGV' <seconds> <command...>`.
- **After each task**, run the checks from the "Testing changes" section of `AGENTS.md` (`cargo test --release`, MQ=4 MS=8 run, `strategy_check`).
- **The search is nondeterministic.** State counts and timings vary between runs. Repeat measurements (the `bench` binary does this) and compare several runs.
- **Parallelism.** The solver runs on about 100–200 threads. Do not add shared atomics or locks that are written in the hot path.
- **Document new flags** in `AGENTS.md` (sections "Running" and "Measuring").
- **Use `--stats-json`** to collect per-depth numbers instead of parsing the console table.
- Do the tasks in order, starting with Task 0. Tasks 1–3 are independent of each other. Report after each one (see "What to report").
- **Task 8 is the current priority.** It is independent of Tasks 1, 4, 5 and 6; do it before them.

## Task 0: fix the winning/total reporting

**Status: done** (commit "Split state counts at the size limit (task 0)"). Check on MQ=4, MS=8, depth 13: 15,975 + 128,942 = 144,917 = old `w_states`; 37,385 + 128,942 = 166,327 = old `t_states`; test `split_counts_add_up`.

Do this first. The later tasks use these numbers.

### Problem

By default, a state at the size limit (`size ≥ MS`) is stored only when it wins. So in the per-depth row:

- `w_states` counts winning states below the limit **and** at the limit,
- `t_states` counts all states below the limit, but only the winning ones at the limit.

The ratio of the two is therefore meaningless, and `delta` is dominated by wins at the limit. In the MQ=5, MS=12 run, about 96% of each depth's new wins are states at the limit.

### Change

Split the counts in `count_stats` (`rust/src/find/search_state.rs`) by whether the key's size is below the limit. The size of a key is `(key.len() + 1) / 2`; `count_stats` needs `max_size` as a parameter. Today that size is computed only with `--features stats`; compute it always.

Report, per depth, in the console row and in the `depth` record of `--stats-json`:

- winning states below the limit, and their change since the previous depth,
- all states below the limit,
- winning states at the limit (stored only for strategy printing).

Keep the entry ("substate") counts, split the same way. With `--store-at-limit`, losing states at the limit are stored too; report their count as well in that mode.

Update the column description in `AGENTS.md` and the `bench` summary if it reads these fields.

### Checks

- The three new counts add up to the old `t_states`, and the two winning counts add up to the old `w_states` (check on MQ=4, MS=8).
- `find::stats_test` still passes with `--features stats`.
- Wherever a later task says "winning states" or "stored states" (the Task 1 table, `--stop-after-quiet` in Task 2), use the below-limit counts. For `--stop-after-quiet`, a depth is quiet when no count changed, at the limit or below.

## Task 1: colour target option, and the reduced-colour experiment

### Goal

Let Builder try to force fewer colours than `2·MQ − 1`, then measure how the minimum board size and win depth change with the colour target, for MQ=4 and MQ=5.

### Change

Add `-c, --colours <C>`: the number of colours Builder tries to force. The Algorithm then has `C − 1` colours. The default is `2·MQ − 1`. Accept `MQ + 1 ..= 2·MQ − 1` (with `C ≤ MQ` Builder wins trivially with one clique).

The colour count is currently the constant `EXPECTED_COLOURS` in `rust/src/state/mod.rs`. It is used in two places:

- `allowed_colours_for_segment` in `rust/src/state/mod.rs`: the Algorithm may answer with colours `0 .. min(EXPECTED_COLOURS − 1, colours_used + 1)`.
- `print_strategy` in `rust/src/print_strategy/mod.rs`: the first line of the strategy file is `MQ C`.

Suggested implementation: store the target in `SearchState` (a `colours: u8` field, set with a builder method like the existing `with_store_at_limit`), and pass it as a parameter to `allowed_colours_for_segment` and to `StateWithMove::outcomes` / `outcomes_with_colours`. Avoid a process-wide global: tests run in parallel in one process and would interfere.

Also:

- write the colour target into the `run` record of `--stats-json`,
- print it in the `Out: ...` header line,
- make sure `bench` passes solver arguments through unchanged (it should already).

### Checks

- With no `-c`, results must match the table in "Background".
- `cxx/strategy_check` reads the colour target from the first line of the strategy file. Confirm it accepts a reduced-colour strategy: run MQ=4 with `-c 6`, and check the output prints `Strategy is correct.`. If the checker rejects reduced targets, stop and report; do not change the checker without asking.

### Experiment

For each pair below, find the smallest MS where Builder wins:

- MQ=3: C = 4, 5
- MQ=4: C = 5, 6, 7
- MQ=5: C = 6, 7, 8

Procedure for one pair:

1. Start at MS = 2 and increase by 1.
2. Run with `-d 40` and a wall-clock cap of 10 minutes per run.
3. Classify each run as:
   - `win` (record the depth),
   - `lose, converged`: the last 5 or more depths added no winning states and no stored states,
   - `lose, not converged`: reached depth 40 while still changing,
   - `timeout`.
4. Stop at the first win, or at the first timeout.
5. Validate every winning strategy with `strategy_check`.

A timeout for MQ=5, C=8 is an acceptable result: report it as "needs the remote machine".

## Task 2: stop a losing search by itself

**Status: done** (commit "Add --stop-after-quiet (task 2)"). A rigorous loss proof (`--verify-loss`, a safe-set traversal) was also built and then dropped for memory; see `OPTIMISATIONS.md`, "Loss verification".

- `--stop-after-quiet <N>` (default: off): end the search with "lose" after `N` consecutive quiet depths, where no state count changed, below the size limit or at it. Without it a losing search only ends at `-d`. The `end` record of `--stats-json` has `stopped_quiet_at`.
- A quiet stop is strong evidence, not a proof: the search could in principle find wins again at a later depth.
- For an MQ=5, MS=12 rerun with `--stop-after-quiet 3`, pass `--capacity 2100000000` (the stored-state count at depth 19), so the table does not double mid-run. The doubling at depth 18 raised RSS from 37 GB to 75 GB. The user starts that run on the remote machine; do not start it yourself.

## Task 3: depth step

**Status: done, removed.** Slower in every case and more memory; see `OPTIMISATIONS.md`, "Depth step".

### Goal

Reduce the number of full passes over the state space.

### Why

The root loop (`for d in 3..=depth` in `find_strategy_root`, `rust/src/find/root.rs`) raises the depth by 1 each time. A losing state is stored as `Losing{d}`; the next iteration asks about `d + 1`, so nearly every losing state is expanded again. Each iteration costs about the same (2 hours at MQ=5, MS=12), whatever the depth. Raising the depth by `k` per iteration may reach the same depth in `k` times fewer passes.

The risk: a state reached first through a long path is searched with a small depth, then searched again when a shorter path reaches it with a larger depth. With step `k` that can happen up to `k` times per iteration, which would cancel the gain. Only measurement can tell.

### Change

- `--depth-step <k>` (default 1): the depth increase per iteration.
- `--depth-step-from <d>` (default 3): depths below `d` still increase by 1.
- The depth is a `u8`; do not overflow near 255.

### Measurement

Use `bench` (see "Measuring" in `AGENTS.md`), 5 runs per variant, with `k` = 1, 2, 3, 4:

- MQ=4, MS=8, 9, 10 (wins): total time to the win.
- MQ=4, MS=7, `-d 30` (loss): total time.
- MQ=5, MS=9, `-d 24`: total time, with a cap of 15 minutes per run. If `k = 1` does not finish within the cap, lower `-d` until it does and use the same `-d` for all variants.

For MQ=5, set `--depth-step-from` to the first depth at which the stored-state count stops growing in the `k = 1` run.

Record for each variant: total time, number of iterations, total `expansion` count, and the `re-searched (shallow)` number from the bench summary.

### Correctness

- Every winning strategy must pass `strategy_check`.
- A stepped run may find its win at a later depth than `k = 1`. That is fine.
- The search treats an `InProgress` state as a loss to cut cycles. That rule was designed for step 1. Do not try to prove it for larger steps; rely on `strategy_check`, and report any run where a stepped search loses where `k = 1` wins. Task 6 examines this rule directly.

## Task 4: measure whether a change-driven search is worth building

Do this only after Tasks 1–3. It is a measurement, not a new solver.

### Idea

Instead of re-expanding every losing state at each depth, a change-driven search would re-check only the parents of states that just became winning. To know whether that is cheaper, we need to generate parents ("un-moves") and count them.

### Step 1: un-move generators

Put them in a new file `rust/src/state/unmoves.rs` (it needs the private `remove_at_index`).

**Insert parents** of a state `T` with limits `(lf, lb)`:

- Forward move: `insert_segment(start, end, colour)` puts the start event at index `start` and the end event at index `end + 1`, leaves `limit_front` unchanged, and adds 2 to `limit_back`.
- So for each segment of `T` whose start index `i` and end index `j` satisfy `i ≥ lf` and `j ≤ lb − 1`: remove events `i` and `j`. The result is a parent `P` with move `Move(i, j − 1)`.
- Normalise `P`. If normalisation mirrors it, mirror the move with `flip_move`.
- Return `(P, move)`. The colour does not need to be returned: the consumer re-checks all colours of that move.

**Front reduction parents** of `T` with `limit_front = j`:

- Forward move: `try_moved_limit_front` removes the first segment (start at index 0, end at the first end event) and sets the limit so that `T.limit_front` equals the number of starts that were inside the removed segment.
- So the parent is `T` with a new end event inserted at index `j` and a new start event inserted at index 0, for each colour `c` that none of the first `j` segments of `T` uses. Colours that do not appear in `T` at all are equivalent; generate one of them, and only if the colour budget allows a new colour.
- Requirements: the first `j` events of `T` are starts, `j + 1 ≤ MQ`, and the parent has at most `MS` segments.
- The parent's `limit_front` is `j + 1` (the most confined choice; less confined ones follow from the barrier rule) and its `limit_back` is `T.limit_back + 2`.
- A `Winning` entry with barrier front `f` applies to every `limit_front ≤ f`, so generate parents for every `j` in `0 ..= f`.

**Back reduction parents**: mirror `T`, take front reduction parents, mirror back.

### Step 2: property tests (these define correctness)

For many states reachable from the start state (for MQ=3 and MQ=4, small MS):

- For every insert move `m` of `S` and every allowed colour: let `T` be the normalised result. `insert_parents(T)` must contain normalised `S` with the matching move.
- For every reduction of `S`: let `T` be the result. The matching reduction-parent list of `T` must contain normalised `S` (compare events; for the limit, the generated parent may be more confined than `S`).

A generator that returns extra parents is acceptable (they only cost time). A generator that misses a parent is a bug.

### Step 3: the count

Add a development flag (for example `--project-worklist`) that runs after the search ends and computes:

- `W`: the sum, over all stored `Winning` entries `T`, of
  - the number of insert parents of `T` whose key is in the map, plus
  - for each reduction parent `R` of `T`: 1 if `size(R) < MS` and its key is in the map; or, if `size(R) = MS`, the number of insert parents of `R` whose key is in the map.
- `A`: the `move_tried` counter summed over all depth iterations (needs `--features stats`).

Both count "one move re-checked". The ratio `A / W` is the projected speed-up of a change-driven search.

Report `A`, `W` and the ratio for MQ=4 MS=7 (`-d 20`), MQ=4 MS=8, and MQ=5 MS=9 (same `-d` and time cap as in Task 3).

If the ratio is below about 3, say so: the rewrite is then not worth it, and the un-move code should be removed again (the crate flags unused code).

## Task 5: allow MS=13

Small and independent, but do it last.

- The map key is `CompressedState<7>` (56 bits). A state needs `5·MS − 4` bits, so MS=13 needs 61 bits: change `COMPRESSED_BYTES` in `rust/src/find/search_state.rs` to 8.
- The slot must stay 12 bytes (8-byte key plus 4-byte value). A test pins the slot size; it must still pass.
- Add a start-up check that `5·MS − 4 ≤ 8·COMPRESSED_BYTES` and that `2·MS ≤ 28` (the event capacity of `State`), with a clear error message, if no such check exists.
- Add a unit test that compresses and decompresses states with 13 segments.
- Compare speed and peak memory before and after with `bench` on MQ=4, MS=8 (5 runs each). There should be no measurable change.

## Task 6: are stored losses too pessimistic?

Independent of Tasks 4 and 5. Do it after Task 3.

### Why

`OPTIMISATIONS.md`, section 1, describes a fixed bug: lookups below a back reduction used to miss the cache. After the fix, wins came about 2 depths later and strategy files grew 2–5×. The old wins passed `strategy_check`, so they were real. A fresh search that finds a win where the cache says "lose" means some stored losses are wrong in the pessimistic direction. Each lost depth costs one full pass (about 2 hours at MQ=5, MS=12).

There are two suspects. Neither is confirmed; this task measures them.

1. **The `InProgress` cut-off.** A state `S` reaches a state `X` that is still being searched higher up the same path. That counts as a loss, and `S` is stored as `Losing{d}`. If `X` later turns out to win, `S` wins too, but the stored loss stays until some path asks `S` for more than `d`.
2. **Reductions below the size limit are searched at depth 1** (`reduction_search_depth` in `find_strategy_inner`, `rust/src/find/mod.rs`). A win of the form "reduce early, then insert several segments" is only found if the reduced state is already stored as winning.

Note that the depth at which the solver reports a win is not the true minimum number of inserts in either direction: a stored `Winning` entry is accepted at any remaining depth, including 0.

### Step 1: an exact reference search

Add a slow, simple, obviously correct search, behind a development flag (for example `--reference`). It is also a useful test oracle, so keep it if it works.

`exact(s, d)`: can Builder win from `s` with at most `d` inserts? With `s` normalised, limits included:

1. If `d == 0`: no.
2. If some reduction `r` of `s` has `exact(r, d)`: yes. (Reductions keep `d`.)
3. If `size(s) ≥ MS`: no.
4. If some insert move has `exact(child, d − 1)` for every allowed colour (also when there is no allowed colour): yes.
5. Otherwise: no.

Requirements:

- No `InProgress`, no cut-offs. The recursion ends by itself: inserts lower `d`, reductions lower the size.
- Memoise per normalised state (limits included) two numbers: the largest `d` known to lose and the smallest `d` known to win. This memo is exact because it does not depend on the path.
- Use the same move generation and colour rules as the solver (`moves`, `allowed_colours_for_segment`, `try_moved_limit_front`, `try_moved_limit_back`).
- A single thread is fine.

With the flag, print the smallest `d` for which `exact(start, d)` holds, trying `d = 1, 2, …` up to `-d`.

Run it for MQ=2 MS=3, MQ=3 MS=5, MQ=4 MS=8 and MQ=4 MS=9, with a 15-minute cap each. The solver's win depths today are 4, 7, 13 and 12.

### Step 2: count pessimistic stored losses

Add a development flag (for example `--check-losses`) that runs after the normal search ends. For every stored `Losing{k}` entry with `k ≥ 1`:

- rebuild the state from the key and the entry's barrier (`limit_front = barrier.front`, `limit_back = len − barrier.back`); this needs a small constructor in the `state` module, because the limit setters are private to it;
- if `exact(s, k)` is true, count it as a violation.

Print the number of losing entries checked and the number of violations, split by `k`.

Also add a development flag `--full-depth-reductions` that makes the solver search reductions below the size limit at the full depth instead of depth 1. With it, the solver's intended meaning of `Losing{k}` is the same as the reference's, so every remaining violation comes from the `InProgress` cut-off.

Run these, each with `RAYON_NUM_THREADS=1` and with the default thread count:

| Case | Variants |
|---|---|
| MQ=3, MS=5, `-d 6` (the last losing depth) | default, `--full-depth-reductions` |
| MQ=4, MS=7, `-d 14` | default, `--full-depth-reductions` |
| MQ=4, MS=8, `-d 12` (the last losing depth) | default, `--full-depth-reductions` |

Also record, for MQ=4 MS=8, 9 and 10 without `-d`: the depth of the win and the total time, with and without `--full-depth-reductions` (5 runs each with `bench`).

How to read the result:

- Violations with `--full-depth-reductions` and 1 thread: the `InProgress` cut-off is confirmed as a cause. Go to Step 3.
- Violations only without `--full-depth-reductions`: the depth-1 reduction rule is the cause. Report the win depth and time of both variants; do not go to Step 3.
- More violations with many threads than with 1: other threads' `InProgress` entries also matter. Report the difference.
- No violations anywhere: the explanation above is wrong. Report that and stop.

### Step 3 (only if Step 2 confirms the cut-off): do not store tainted losses

Add a development flag `--exact-losses`:

- Pass the current search path down the recursion as a linked list on the stack (state key and level; each node points to its parent). Do not use a thread-local stack: rayon workers switch between subtrees.
- When a lookup returns `InProgress`, find the shallowest node on the path with the same key and remember its level. If no node on the path has that key, the entry belongs to another thread: remember "unresolvable".
- A losing result carries the shallowest level it depended on. Combine it over the reductions and over the refuting reply of every move. The moves run in parallel, so collect it in a per-node local value (for example a local atomic minimum); do not use a global.
- At a node of level `l`: if the result depends only on levels `≥ l`, store `Losing{depth}` as today. Otherwise store `Losing{0}` (always true) and pass the taint up.
- Wins are never tainted. States at the size limit are not stored; they only pass the taint up.

Checks:

- `--check-losses` reports zero violations with `--exact-losses --full-depth-reductions` and 1 thread, on the three cases of Step 2.
- All winning strategies pass `strategy_check`.

Then measure with `bench`, 5 runs each, on MQ=4 MS=8, 9, 10 and MQ=5 MS=9 (same `-d` and time cap as in Task 3): depth of the win, total time, total expansions, for the default and for `--exact-losses`. Tainted states are searched again on every visit, so the run may be much slower; a result of "exact but slower" is a valid outcome.

## Task 7: does a larger board help Builder from any position?

**Status: done** (`--win-levels`, `rust/src/win_levels.rs`). The empty state's level is 3, 5, 8. The cross-check against the solver passes for MQ=3 MS=5 (test) and MQ=4 MS=8 (ignored test). Runs: MQ=2 N=7, MQ=3 N=9–11, MQ=4 N=8–10; the largest took 165 s and 6.4 GB. Smallest `B0` with no counterexamples: 3 / 5 / 8. **But Table B is vacuous for every `B0 ≥ level(empty)`:** every state reduces step by step to the empty state, so `level(s) ≤ max(size(s), level(empty))` (checked on every run), and every window of size `B0 ≥ level(empty)` has level `B0`. So the result is "smallest `B0` = level of the empty state" by construction. It cannot tell `2·MQ` from `3·(MQ−1)+1`. It only refutes `2·(MQ−1)+1` for MQ=4, where B0 = 7 has counterexamples. Table A is likewise zero above `n = level(empty)`. The tool is not vacuous for MQ=5, where the empty state has no level; Task 8 runs it there.

Independent of all other tasks. It adds a separate analysis mode and does not change the search.

### Why

The plan for proving that 8 colours suffice for MQ=5 is:

1. show by computer that Builder loses on a board of some size `B0`,
2. argue that the Algorithm can apply that result locally on an unbounded board: keep every window of `B0` consecutive segments safe, and answer each new segment by looking only at the segments around it.

Step 2 needs a lemma about **all positions**, not just the empty board:

> **Window property for `B0`.** If Builder wins from a position `T` on a larger board, then Builder already wins on a board of size `B0` from some window of `B0` consecutive segments of `T`.

Three candidate values are on the table:

| Candidate | MQ=2 | MQ=3 | MQ=4 | MQ=5 | Status |
|---|---|---|---|---|---|
| `2·(MQ−1)+1` | 3 | 5 | 7 | 9 | Already refuted as a general rule: at MQ=4 Builder loses on board 7 and wins on board 8. |
| `2·MQ` | 4 | 6 | 8 | 10 | Consistent with the small cases (Builder first wins at 3, 5, 8). For MQ=5 the loss on board 10 is already verified (`--verify-loss`, safe set of 44.2M states). |
| `3·(MQ−1)+1` | 4 | 7 | 10 | 13 | Consistent with the small cases. Board 13 for MQ=5 is not computed. |

The window property for a smaller `B0` implies it for every larger one, so `2·MQ` is the stronger claim. For MQ=5 the property cannot be checked by computer (it needs a board of `2·B0 − 1` segments: 19 or 25). For MQ=2, 3, 4 it can, even though Builder wins there. This task measures it, to see which formula for `B0`, if any, survives the small cases.

### Definitions

- **Game with bound `n`:** the solver's rules with `MS = n`. A state with fewer than `n` segments allows inserts and reductions; a state with `n` segments allows only reductions.
- **Level of a state `s`:** the smallest bound `n ≥ size(s)` at which Builder wins from `s`, with no depth limit. Level 0 means "not winning at any bound up to `N`", where `N` is the largest bound computed.
- **Universe:** every state reachable from the empty state by inserts (with every allowed colour, not only good ones) and reductions, at bound `N`. States are normalised and include their limits.
- **Window of `T`:** for `size(T) > B0`, drop `i` segments from the front with `try_moved_limit_front`, then `size(T) − B0 − i` from the back with `try_moved_limit_back`, for each `i` in `0 ..= size(T) − B0`. Skip a window if a reduction is not possible.

### Algorithm

Add a flag (for example `--win-levels`) that uses `-q` for MQ and `-s` for `N`, runs this analysis instead of the search, and writes a report.

**1. Enumerate the universe.** Breadth-first from the empty state, in parallel:

- a state with fewer than `N` segments: generate every insert move with every allowed colour, and both reductions;
- do not store states with exactly `N` segments. When one is generated, add its two reductions to the frontier and discard it. (They are most of the universe, and their result follows from their reductions.)

After the enumeration the set of states is fixed. A memory-lean layout is a sorted `Vec` of states with a parallel `Vec<AtomicU8>` of levels, looked up by binary search.

**2. Compute levels.** All levels start at 0.

```
for n in 1 ..= N:
    repeat until a full sweep changes nothing:
        for every stored state s with size(s) ≤ n and level(s) == 0   (in parallel):
            if wins(s, n): level(s) = n

wins(s, n):
    some reduction r of s has won(r)
    or ( size(s) < n and some insert move m of s has won(child) for every allowed colour;
         a move with no allowed colour counts )

won(x):
    if size(x) == N:  some reduction of x has level != 0      (x is not stored)
    else:             level(x) != 0
```

Levels only ever go from 0 to a value, so parallel sweeps with stale reads are safe: a stale read can delay a win to the next sweep, never create a wrong one. This is an analysis tool, so atomics here are fine.

**3. Report.**

- **Universe size** per segment count.
- **Level of the empty state.** It must be 3, 5, 8 for MQ = 2, 3, 4 (when `N` is at least that).
- **Table A, "extra board helped":** for each `n` in `2 ..= N`, the number of states with `level == n` and `size ≤ n − 1`, split by size. These are positions lost at bound `n − 1` and won at bound `n`.
- **Table B, window property:** for each `B0` in `MQ ..= N − 1`:
  - take every stored `T` with `B0 < size(T) ≤ min(N − 1, 2·B0 − 2)` and `0 < level(T) ≤ min(N, 2·B0 − 1)`;
  - `T` is a counterexample if none of its windows has `level == B0`;
  - print the number of `T` checked, the number of counterexamples, and the smallest counterexample (as a state string).

### Checks

- The level of the empty state matches 3, 5, 8.
- Cross-check with the normal solver: after a normal run with MQ=3, MS=5, and one with MQ=4, MS=8, every state the solver stored as `Winning` must have `0 < level ≤ MS` in an analysis with `N = MS`. Add this as a test for MQ=3.
- For MQ=2 with `N = 4`, the universe is tiny: print all states with their levels and check a few by hand against `cxx/2_3.strategy`.

### Runs

| MQ | `N` | Cap |
|---|---|---|
| 2 | 7 | none needed |
| 3 | 9, then 10 | 20 minutes each |
| 4 | 8, then 9, then 10 | 30 minutes each; record peak memory |

If a run hits its cap or runs out of memory, report the universe size reached and stop; larger `N` goes to the remote machine, and the user starts those runs.

### How to read the result

- A counterexample for some `B0` means that window size is too small for that MQ, whatever the formula says.
- No counterexamples for a `B0` only means "not refuted up to `N`".
- `B0` below the level of the empty state (3, 5, 8) is expected to fail; the interesting rows are from that value upwards.
- Report, per MQ, the smallest `B0` with no counterexamples, next to `2·(MQ−1)+1`, `2·MQ` and `3·(MQ−1)+1`, and the largest `n` with a non-zero row in Table A.
- The row `B0 = 2·MQ` matters most. A full check of it needs `N = 4·MQ − 1`: 7 for MQ=2 (reachable), 11 for MQ=3 (try it; report how far you get), 15 for MQ=4 (out of reach, so that row will be partial).

## Task 8: a certificate that the Algorithm survives on an unbounded board

Independent of Tasks 1–6. It reuses the universe and levels of Task 7 (`rust/src/win_levels.rs`) and does not change the search.

### Why

The statement we want for MQ=5 is: *if Builder can force 9 colours at all, it can force them on a board of at most 12 segments.* Together with a computed loss on board 12 this gives "8 colours suffice". But for MQ=5 that statement is equivalent to "8 colours suffice", so it needs its own proof. Task 7 showed that the small cases (MQ ≤ 4) cannot test it: Builder wins there, so every position wins.

This task builds a proof that a computer can check. Fix a window size `k` and a set `F` of states.

- A **board** is a state with open limits (`limit_front = 0`, `limit_back = len`) and any number of segments. Boards can be larger than a `State` can hold; the check never builds them.
- The **views** of a board with `m` segments: the board itself if `m ≤ k`; otherwise, for each `a` in `0 ..= m − k`, the state after `a` front reductions and `m − k − a` back reductions (the `windows` function of `win_levels.rs`).
- A board is **good** if every view of it is in `F`.

> **Claim.** If (1) the empty board is good, and (2) for every good board and every legal insert there is a colour after which the board is still good, then the Algorithm never needs colour `2·MQ − 1` on an unbounded board: it keeps the board good forever.

The claim holds for any `F`, however it was built. So a program that confirms (1) and (2) for some `F` proves the result. Condition (2) looks like it needs boards of `2k − 1` segments, but a new segment only changes the at most `k` views that contain it, and those overlap in a chain. Step 3 walks along such chains and never builds the whole board.

**Soundness rule for this task:** the checker may report a failure that no real board produces, but it must never report "pass" when a good board and an insert without a good colour exist. When in doubt, explore more, not less.

Write `NC = 2·MQ − 2` for the number of Algorithm colours.

### Step 1: is "safe on board n" stable? (no new code)

Run the existing analysis on MQ=5:

| Run | Cap |
|---|---|
| `--win-levels -q 5 -s 8` | 30 minutes |
| `--win-levels -q 5 -s 9` | 30 minutes |

Record universe size per segment count, time and peak memory, and the growth factor from `N = 8` to `N = 9`. Run `-s 10` only if that factor predicts under 30 minutes and under half of the machine's memory; otherwise it goes to the remote machine, and the user starts it.

Report "States per level" and Table A in full. Table A, row `n`, column `s` counts states with `s` segments that are lost on board `n − 1` and won on board `n`. The level of the empty state must be 0 in all these runs (Builder loses on board 10).

Stop and report after this step, then continue with Step 2.

### Step 2: a reference checker (this defines correctness)

A slow, direct check for tiny cases, used only in tests:

1. Enumerate every board with at most `S` segments (breadth-first from the empty board, every insert, every colour of `outcomes()`, normalised).
2. Mark the good ones.
3. For every good board with fewer than `S` segments and every insert move: the set of colours after which the child is good. An empty set is a **failure**.

`F` is passed as a membership function on canonical states (use `canonical` from `win_levels.rs`). A state that is not in the universe is not in `F`.

### Step 3: the chain checker

Add a mode `--window-closure` with `-q` = MQ, `-s` = `M` and `--window <k>` (default `M − 1`). It uses

> `F` = every state of the universe at bound `M` with at most `k` segments and level 0.

Require `MQ ≤ k ≤ M − 1` and `k + MQ ≤ 14` (the largest piece has `k + MQ` segments; a `State` holds 14). Exit with a clear message otherwise.

The verdict has three parts:

1. **Empty board:** the empty state is in `F`.
2. **Small boards:** for every board in `F` with fewer than `k` segments, every insert has a colour whose child is in `F`.
3. **Chains:** boards with at least `k` segments, below.

**New state operations** (in the `state` module; the limit setters and `remove_at_index` are private to it):

- `append_back(S, c, l)`: put back a segment that a back reduction removed. `S` has `n` events and `limit_back = b`.
  - Requires: `n − b + 1 ≤ MQ` (the new segment meets the `n − b` segments that end after index `b`); `c` is not the colour of any of those segments; `b + 1 ≤ l ≤ n + 2`; `n + 2 − l ≤ MQ − 1`.
  - Events: `S[0..b]`, start of `c`, `S[b..n]`, end of `c`. `limit_front` is unchanged; `limit_back = l`.
  - `l = n + 2` means that nothing further to the right touches the piece.
- Remove the segment of a given rank (rank = number of segments that start before it), keeping the limits consistent the way `remove_at_index` does.
- Set the colour of the segment of a given rank.

**The walk.** A *piece* `P` is a state that holds some consecutive old segments and, once inserted, the new segment `x`. `L` and `R` are the numbers of old segments before and after `x`. The colour stored for `x` is meaningless: `append_back` must ignore `x` in its colour condition, and so must the deduplication below. `ok` is the set of colours still acceptable for `x`.

```
for every member V of F with k segments, in both orientations (V and its mirror image):
  for a in {k}, and also in 0 ..= k−1 if V.limit_front == 0:
    P = V;  L = a;  R = k − a;  ended = false
    // 1. room on the right, so that x cannot reach past the back limit
    while R < MQ − 1 and not ended:  extend
    // 2. the new segment
    for every insert move of P whose new segment has exactly `a` segments before it:
        ok = the colours in 0..NC that no neighbour of the new segment uses
        if ok is empty: FAIL
        insert the segment as x
        // 3. slide the window over x, from "x last" to "x first"
        for i = min(a, k−1) down to 0:
            need = k − 1 − i                      // old segments after x in this window
            while R < need + 1 and not ended:  extend
            if R < need: this chain is finished without failure
            for each c in ok:
                W = P with x coloured c, front-reduced L − i times, back-reduced R − need times
                if W is not in F: remove c from ok
            if ok is empty: FAIL
            front-reduce P until L == i

extend:    // branches; each branch continues separately
    if limit_back(P) == len(P): one branch sets ended = true and changes nothing else
    for every (c, l) that append_back allows:
        P' = append_back(P, c, l);  R += 1
        O = P' without x, front-reduced until k segments remain
        keep this branch only if O is in F
```

Notes:

- `c` ranges over all `NC` colours in `append_back`, not only "used + 1": the piece is a fragment, and a colour missing from it may be in use elsewhere.
- A piece with `limit_back < len` must be extended (a neighbour exists). Only a piece with `limit_back == len` may end.
- Without deduplication the walk is exponential. Memoise on (piece, position of `x`, `i`, `ended`, `ok`), with colour names renamed in order of first appearance among the old segments (do **not** mirror), `ok` renamed the same way, and the colours absent from the piece sorted so that those in `ok` come first. Two walk states that differ only by a permutation of colour names must be one entry; add a test for that.
- A depth-first walk per seed with a shared memo ("explored, no failure below") gives a witness for free: the stack at the first failure.
- This is an analysis tool: atomics and concurrent maps are fine. Run seeds in parallel.
- If you find a simpler formulation that satisfies the soundness rule and passes the tests below, use it and say so in the report.

**Output:** the size of `F` per segment count; the three verdicts; walk states visited; the number of (seed, move) starts from which a failure is reachable, split by the step `i` at which `ok` became empty; one witness (the pieces from the seed to the failure, as state strings, with the position of `x`); lookups that missed the universe; time and peak memory.

### Tests

1. **`append_back` round trip.** For every state `T` of a small universe with a back reduction `S`: `T` is among the results of `append_back(S, ·, ·)`, and every result reduces back to `S`.
2. **Follow a real board.** For MQ=2 and MQ=3, small `k`, and `S = 2k + 2` in the reference checker: for every good board `T` and every insert, run the walk steered along `T` (the pieces follow from `T` plus the new segment by reductions). At every `extend`, the piece that `T` dictates must be among the generated branches. The final `ok` must equal the reference checker's colour set.
3. **Reference failure implies walk failure.** On the same cases: whenever the reference checker finds a failure, the chain verdict must be "fail". Use these families: all universe states with at most `k` segments; the level-0 states; and each of those with a random half of its `k`-segment members removed (fixed seed).
4. **Witness replay.** A separate function recomputes a reported witness step by step and confirms that `ok` ends empty.
5. **Cases that must fail** (Builder wins, so no certificate can exist): MQ=3 with `M = 4`, `k = 3`; MQ=4 with `M = 7`, `k = 6` (cap 30 minutes; make it an ignored test if slow).

### Runs

| MQ | `M` | `k` | Cap |
|---|---|---|---|
| 3 | 4 | 3 | none needed |
| 4 | 7 | 6 | 30 minutes |
| 5 | 8 | 7 | 60 minutes |
| 5 | 9 | 7, then 8 | 60 minutes each |

If a run hits its cap or runs out of memory, report the walk states visited and the memory used, and stop. `M = 10` (with `k = 8` and `k = 9`) goes to the remote machine; the user starts it.

### How to read the result

- A "pass" on MQ=5 would be a proof that 8 colours suffice. Treat it with suspicion first: re-run the tests, check the witness-free result on a second `k`, and report it without claiming the result.
- A "fail" is the expected outcome for these families. The useful output is how many starts fail, at which step, and what the witness looks like. Step 1 predicts it: if Table A shows boards of at most `k` segments that are lost on board `M` and won on a larger one, the family for that `(k, M)` cannot pass.
- Removing views from `F` until the check passes is the follow-up. It is not part of this task; report first.

## Not in scope (ask the user first)

- Writing the change-driven (worklist) solver itself. It depends on the Task 4 result.
- A denser hash table or other memory work for MS=13. Projection: about 2.8·10¹⁰ entries; at today's 21 bytes per entry that is about 600 GB against 400 GiB available.
- Saving the map to disk and loading it again.
- Any MQ=5 run beyond the capped ones listed above.
- Pruning the family `F` of Task 8 until the check passes.
- Changes to the C++ tools.

## What to report

After each task, a short report with:

- what changed (files and flags),
- the test results from "Testing changes" in `AGENTS.md`, with the actual output of failing commands if any,
- the task's table:
  - Task 1: one row per (MQ, C, MS) with result, win depth, winning states, stored states, time;
  - Task 2: the depth at which each quiet stop happened;
  - Task 3: one row per (case, `k`) with total time, iterations, expansions, shallow re-searches;
  - Task 4: `A`, `W`, ratio per case;
  - Task 5: time and peak memory before and after;
  - Task 6: the exact win depths from the reference search; violations per case, variant and thread count; win depth and time per variant;
  - Task 7: per (MQ, `N`): universe size, level of the empty state, Table A, Table B, time and peak memory;
  - Task 8: Step 1: per `N`, universe size, states per level, Table A, time and peak memory. Step 3: per (MQ, `M`, `k`), the size of `F`, the three verdicts, walk states visited, failing starts by step, one witness, time and peak memory;
- anything that did not match the expectations in this file. Do not adjust a check to make it pass; report the mismatch instead.
