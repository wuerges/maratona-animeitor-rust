# Scale live and revelation views to 5,000 teams

## Summary

Reduce rendering work to the visible scoreboard region while retaining complete competition data and scoring. Support live updates and revelation, preserving visible ranking animations and team media.

## Changes

- Replace eager scoreboard rendering with a keyed viewport window and one viewport of overscan above and below. Preserve full scroll height, group filtering, remote scrolling, and revelation centering. Read actual row height and recompute on resize.
- Keep visible rows mounted during ranking transitions. Animate distant entrants from the nearest viewport boundary; retain departing visible rows until their animation finishes. Retarget transitions on newer updates and clean up retained rows afterward.
- Use transforms for scoreboard movement with the existing five-second easing. Leave submission-panel animation behavior unchanged.
- Move team photos and audio into one selected-team overlay independent of mounted rows. Preserve keyboard controls, autoplay, volume, and off-screen selection.
- Share and memoize placement filtering and lookup. Only mounted rows subscribe to presentation state; retain scoring and correction semantics.
- Add bounded revelation checkpoints: retain the starting state and up to eight snapshots at 100-step intervals. Backward navigation restores the closest earlier checkpoint and replays remaining steps. Reset clears checkpoints.

## Validation

- Establish release-build browser baselines at 2,000 and 5,000 teams using deterministic fixtures with 15 problems and 100,000 submissions, including frozen results and corrections.
- Measure initial display, scrolling, filtering, live batches, reveal steps, backward navigation, and bulk reveal. Record browser, hardware, memory, long tasks, and mounted row counts.
- Target p95 ordinary update/reveal processing under 100 ms and backward navigation under 200 ms on the recorded reference machine; aim for smooth 60 Hz scrolling. Report measurements separately from targets.
- Verify mounted rows are bounded by viewport, overscan, and active transitions, with no accumulated nodes or listeners.
- Test distant entrants, visible reorderings, interrupted animations, filters, resize, remote scrolling, media, and offline revelation.
- Compare scoring and checkpoint replay against existing behavior, including ties, first solves, corrections, reset, and jumps. Run model tests and the release WebAssembly build.

## Assumptions

- Visible animations matter; distant teams entering view may start at the viewport boundary instead of rendering their entire off-screen journey.
- No server API, authorization, or wire-format changes are planned. No API version bump or snapshot update is required.
- Code inspection identifies likely bottlenecks, but browser measurements must establish their relative cost and whether further scoring or worker optimizations are needed.
