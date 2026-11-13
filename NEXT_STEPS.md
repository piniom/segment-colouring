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
2. a rigorous "Builder loses on this board size" result (Task 2),
3. faster runs (Tasks 3 and 4),
4. the ability to run MS=13 (Task 5),
5. a check of whether the search reports "lose" at depths where a win exists (Task 6).

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

## Task 0: fix the winning/total reporting

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

## Task 2: verify a loss

### Goal

When a search ends with "lose", prove that Builder really cannot win on this board size. Today a loss only means "no win found up to this depth". The proof must not depend on the search's internals (depths, `InProgress`, stored losing entries).

### Definition

A set `S` of states (with limits) is a **safe set** when it contains the empty start state and every state `s` in `S` satisfies:

1. No insert move of `s` leaves the Algorithm without an allowed colour.
2. For every insert move of `s`, at least one allowed colour leads to a state in `S`.
3. Every reduction of `s` (front and back, when possible) is in `S`.

If `size(s) ≥ MS`, there are no insert moves, so only rule 3 applies. This matches the search: at the size limit Builder can only reduce.

If a safe set exists, the Algorithm survives forever on boards of size at most `MS`.

### Algorithm

Build `S` by a traversal from the start state:

- Keep a concurrent visited set of normalised states (limits included), and a frontier. Process the frontier in parallel, layer by layer. Do not recurse deeply.
- Define `marked_win(s)`: the map says `Winning` for the normalised `s` (`get_knowledge`), or `marked_win` holds for one of its reductions. The recursion follows reductions only, so its depth is at most `size(s)`.
- When visiting `s`:
  - add both reductions to the frontier,
  - for each insert move: if it has no allowed colour, fail; otherwise pick the first colour whose resulting state is not `marked_win` and add that state to the frontier; if every colour is `marked_win`, fail.
- Fail immediately if the start state is `marked_win`.

`marked_win` only guides which colour to pick. The proof rests on the three rules being checked for every visited state, so a wrong map entry can cause a failure but never a false "verified".

On failure, print the state and the move. On success, print the number of states visited.

### Flags

- `--verify-loss`: after the search ends with a loss, run the verification and print the result. Add the outcome and the visited count to the `end` record of `--stats-json`.
- `--stop-after-quiet <N>` (default: off): end the search with "lose" when `N` consecutive depths added no winning states and no stored states. Without this, a losing search only ends at depth 255. The per-depth counts are currently computed in the printer thread (`spawn_printer` in `rust/src/find/root.rs`); reuse them rather than scanning the map twice.

### Checks

Add these as tests where they are fast enough, otherwise run them by hand:

| Run | Expected |
|---|---|
| MQ=4, MS=7, `-d 20 --verify-loss` | verified |
| MQ=4, MS=6, `-d 10 --verify-loss` | verified |
| MQ=3, MS=4, `-d 10 --verify-loss` | verified |
| MQ=4, MS=8, `-d 10 --verify-loss` (Builder wins at depth 13, search cut short) | verification fails |
| MQ=3, MS=5, `-d 5 --verify-loss` (Builder wins at depth 7, search cut short) | verification fails |
| MQ=4, MS=7, `--stop-after-quiet 3 --verify-loss` | stops by itself, verified |

The two "fails" rows are the important ones: they show the verifier cannot be fooled by an unfinished search.

Memory note: the visited set may be large for MQ=5. Do not optimise it yet; report the visited count compared with the number of stored states.

### Why this matters for MQ=5, MS=12

- The running MS=12 process cannot be verified afterwards: several depths with no new wins are convincing, but they are not a proof. A rigorous loss needs a rerun with `--stop-after-quiet 3 --verify-loss`. The user starts that run on the remote machine; do not start it yourself.
- For that rerun, pass `--capacity 2100000000` (the stored-state count at depth 19), so the table does not double mid-run. The doubling at depth 18 raised RSS from 37 GB to 75 GB.
- The visited count is the size of one actual Algorithm strategy on this board. If it is far smaller than the stored-state count (about 2·10⁹), the strategy is small enough to inspect for structure, which the argument for unbounded boards needs. Always print it, and print it per state size as well.

## Task 3: depth step

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
- Every loss must pass `--verify-loss` from Task 2.
- A stepped run may find its win at a later depth than `k = 1`. That is fine.
- The search treats an `InProgress` state as a loss to cut cycles. That rule was designed for step 1. Do not try to prove it for larger steps; rely on the two checks above, and report any run where a stepped search ends with a loss that fails verification. Task 6 examines this rule directly.

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

## Not in scope (ask the user first)

- Writing the change-driven (worklist) solver itself. It depends on the Task 4 result.
- A denser hash table or other memory work for MS=13. Projection: about 2.8·10¹⁰ entries; at today's 21 bytes per entry that is about 600 GB against 400 GiB available.
- Saving the map to disk and loading it again.
- Any MQ=5 run beyond the capped ones listed above.
- Changes to the C++ tools.

## What to report

After each task, a short report with:

- what changed (files and flags),
- the test results from "Testing changes" in `AGENTS.md`, with the actual output of failing commands if any,
- the task's table:
  - Task 1: one row per (MQ, C, MS) with result, win depth, winning states, stored states, time;
  - Task 2: the six checks with their outcomes, and visited counts;
  - Task 3: one row per (case, `k`) with total time, iterations, expansions, shallow re-searches;
  - Task 4: `A`, `W`, ratio per case;
  - Task 5: time and peak memory before and after;
  - Task 6: the exact win depths from the reference search; violations per case, variant and thread count; win depth and time per variant;
- anything that did not match the expectations in this file. Do not adjust a check to make it pass; report the mismatch instead.
