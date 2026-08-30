---
name: leptos-use
description: Reference for leptos_use 0.19 (the Leptos port of VueUse, pinned 0.19.0, targets leptos 0.8) — reactive utilities for timers, DOM events, media queries, storage, network and misc state, with signatures, feature flags and confirmed gotchas. Use when the client needs debouncing, intervals, event listeners, media queries, localStorage or similar reactive browser utilities.
---

# leptos-use 0.19 (pinned: 0.19.0) — CSR reference

Collection of Leptos utilities inspired by VueUse. Targets **leptos 0.8**; all signatures below verified against the local 0.19.0 source. Everything that touches the DOM/browser is wasm-only at runtime (panics natively); pure-reactive helpers work anywhere.

## Dependencies & features

`leptos-use = "0.19"` — default features include nearly every `use_*` function. Add explicitly only when needed:

- `math` — reactive math module (needs `num`)
- `reactive_stores` — store integration
- `ssr` / `axum` / `actix` — SSR stubs / header reading (this repo is CSR: **leave off**)
- `element` — `use_document`/`use_window` + `IntoElementMaybeSignal` traits alone
- `is` — the `IS_IOS` static
- storage/cookie/websocket/broadcast_channel need a `codee` codec: `codee = { version = "0.3", features = ["json_serde", "base64"] }` (e.g. `JsonSerdeCodec`, `FromToStringCodec`)

Option structs use `DefaultBuilder` style: `UseIntervalOptions::default().immediate(false)`.

## Timers & scheduling

| Function | Signature / usage |
|---|---|
| `use_interval(ms)` | `UseIntervalReturn { counter: Signal<u64>, reset, pause, resume, is_active }` — **auto-starts** (`immediate: true`) |
| `use_interval_fn(cb, ms)` | `Pausable { is_active, pause, resume }` — auto-starts; reactive interval signal re-arms it; interval `0` is ignored |
| `use_timeout_fn(cb, delay)` | `UseTimeoutFnReturn { is_pending, start(arg), stop }` — **never auto-starts**; delay is **f64 ms**; re-`start` cancels the previous |
| `use_raf_fn(cb)` | `Pausable` — cb gets `UseRafFnCallbackArgs { delta, timestamp }`; auto-starts |
| `use_timestamp()` | `Signal<f64>` — ms since epoch, refreshed per frame (or `TimestampInterval::Interval(u64)`) |
| `use_debounce_fn(f, ms)` | returns the **debounced closure**: `let debounced = use_debounce_fn(f, 500.0); debounced();` — result comes back as `Arc<Mutex<Option<R>>>` (`.lock().unwrap().take()`); args need `_with_arg` variant; `max_wait` via `_with_options` |
| `use_throttle_fn(f, ms)` | same shape; `ThrottleOptions { leading, trailing }` |
| `signal_debounced(src, ms)` | `Signal<T>` derived from a source signal (also `_local`, `_with_options`) |
| `signal_throttled(src, ms)` | `Signal<T>` derived |

Note the numeric types: interval is **u64 ms**, timeout/debounce delays are **f64 ms**.

## State & reactive helpers

- `use_toggle(true)` → `{ toggle, value, set_value }`
- `use_cycle_list(vec![...])` → `{ state, set_state, index, next, prev, shift }` — cyclic next/prev
- `use_sorted(list)` / `use_sorted_by` / `use_sorted_by_key` → `Signal<I>` re-sorted reactively
- `sync_signal(left, right)` — two-way sync between any read/write signal pair; `SyncSignalOptions { immediate, direction, transforms/assigns }`
- `watch_with_options(deps, cb, opts)` → stop-fn; `WatchOptions` builder has `.debounce(ms)` / `.throttle(ms)`; shorthands `watch_debounced` / `watch_throttled` / `watch_pausable` / `whenever` (fires when value is truthy)
- `use_supported(|| bool)` → `Signal<bool>` feature detection (read `.get()`; `false` under SSR)
- `is_err/is_none/is_ok/is_some` predicates
- math (`leptos_use::math::*`, feature `math`): `use_abs/ceil/floor/round`, `use_min/use_max`, `use_and/use_or/use_not` — pure `Signal::derive` wrappers

## DOM, sensors & browser APIs

- `use_event_listener(target, leptos::ev::click, |evt| …)` — target may be an element, `NodeRef`, or a **signal** of either (re-registers when the signal changes); returns a stop closure; auto-cleans on owner cleanup; options `.capture/.once/.passive`. Handlers run in a `SpecialNonReactiveZone` in debug builds.
- `use_window_size()` → `{ width, height: Signal<f64> }` (SSR: `INFINITY` — check `is_finite()`)
- `use_media_query("(prefers-color-scheme: dark)")` → `Signal<bool>` — query may be a signal
- `use_preferred_dark()` / `use_preferred_contrast()` / `use_prefers_reduced_motion()`
- `use_document_visibility()` → `Signal<web_sys::VisibilityState>`; `use_window_focus()` → `Signal<bool>`
- `use_window()` / `use_document()` — SSR-safe newtype wrappers (`.body()`, `.navigator()`, …; `Option`-safe)
- `use_mouse()` → `{ x, y, source_type }`; `use_mouse_in_element(target)` adds element-relative fields
- `use_scroll(target)` → `{ x, y, is_scrolling, arrived_state, directions }`; `use_window_scroll()` → `(Signal<f64>, Signal<f64>)`
- `use_element_size(target)` / `use_element_bounding(target)` / `use_element_hover(el)` / `use_element_visibility(target)` (IntersectionObserver)
- `use_resize_observer` / `use_mutation_observer` / `use_intersection_observer` — callbacks take `(Vec<Entry>, Observer)`
- `use_active_element()` → `OptionLocalSignal<web_sys::Element>`
- `use_device_pixel_ratio()` → `Signal<f64>`
- `use_breakpoints(breakpoints_tailwind())` → `.ge/.gt/.le/.lt(BreakpointsTailwind::Md) -> Signal<bool>`, `.between(min,max)`, `.current()`; presets: tailwind, bootstrap_v5, material, ant_design, quasar, semantic, master_css
- `use_idle(timeout_ms)` → `{ idle, last_active, reset }`
- `use_infinite_scroll(el, on_load_more)` → `Signal<bool>` loading
- `use_draggable(target)` → `{ x, y, is_dragging, style, set_position }`; `use_drop_zone(target)` → `{ files, is_over_drop_zone }`
- `use_textarea_autosize(target)` → `{ content, set_content, trigger_resize }`
- `on_click_outside(target, handler)` — with `.ignore(...)` option

## Storage & persistence

```rust
// needs a codee codec, e.g. codee's JsonSerdeCodec / FromToStringCodec
let (state, set_state, remove) = use_local_storage::<MyState, JsonSerdeCodec>("my-state");
let (flag, set_flag, _) = use_session_storage::<bool, FromToStringCodec>("my-flag");
```

- `use_storage(StorageType, key)` — `StorageType::{Local, Session, Custom}`; key may be a signal; `UseStorageOptions { initial_value, on_error, listen_to_storage_changes, filter }`. **Errors are silently swallowed by default** — set `.on_error(...)` to observe (private mode etc.). Same-key instances sync across tabs via a custom event.
- `use_cookie::<T, C>(name)` → `(Signal<Option<T>>, WriteSignal<Option<T>>)` — same-name calls sync; external `document.cookie` changes are NOT observed; SSR needs the `axum`/`actix` feature.
- `use_color_mode()` → `{ mode, set_mode }` — `ColorMode::{Light, Dark, Auto, Custom}`; persists to localStorage + `class` attribute; `.attribute("theme")` customizes.
- `use_css_var("--x")` → `(ReadSignal<String>, WriteSignal<String>)`; `use_favicon()`.

## Network & devices

- `use_websocket::<Tx, Rx, C>(url)` → `{ ready_state, message, open, close, send }` — note: this repo's client-sdk has its own ws layer; use this only for standalone features
- `use_event_source::<T, C>(url)` → `{ message, ready_state, error, open, close }` — auto-connect + auto-reconnect (`reconnect_limit: 0` disables)
- `use_broadcast_channel::<T, C>(name)` → `{ is_supported, message, post, close }`
- `use_clipboard()` → `{ is_supported, copied, copy }`; `use_geolocation()` → `{ coords, error }`; `use_permission(name)`; `use_web_notification()`
- `use_user_media()` / `use_display_media()` → `{ stream, start, stop, enabled }`
- `use_screen_orientation()` / `use_device_orientation()`; `use_service_worker()`; `use_web_lock(name)` (needs unstable web-sys APIs)
- i18n/format: `use_intl_number_format(opts)`, `use_intl_datetime_format(opts)` → `.format(signal) -> Signal<String>`; `use_locales()` / `use_locale(list)`; `use_calendar()`

## Confirmed gotchas (0.19.0 source)

1. **No VueUse names** `use_previous`, `use_signal_interval`, `use_ref_history`, `use_focus`, `use_fullscreen`, `use_network`, `use_battery` — absent from the crate. Use `use_interval` + a signal, or hand-roll.
2. **Returned closures (pause/resume/stop/copy/…) are sendwrapped** — type-level `Send + Sync` but must be called on the same thread; cross-thread panics.
3. **Timer cleanup**: everything registers `on_cleanup`; but a debounce/throttle callback is **dropped if the component is cleaned before it fires**.
4. **`use_timeout_fn` never auto-starts** — call `start(arg)`. `use_interval`/`use_interval_fn`/`use_raf_fn` do auto-start.
5. **Debounce return quirk**: calling the debounced fn returns `Arc<Mutex<Option<R>>>` — take the value with `.lock().unwrap().take()`.
6. Callbacks run outside the reactive owner tree — don't rely on reactive context inside `use_event_listener`/timer callbacks.
7. Storage reads fall back to `default` and writes drop silently on error unless `.on_error` is set.
8. Value types stored in `Signal` need `Send + Sync`; `web_sys` objects go through `OptionLocalSignal` / `*_local` variants.
