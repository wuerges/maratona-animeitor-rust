---
name: leptos
description: Reference for Leptos 0.8 (pinned 0.8.20, CSR only) as used in animeitor-client — signal/reactive primitives, the #[component] macro and props, view! macro semantics (reactive closure children, static expressions), Show/Suspense/Effect/Resource, and confirmed 0.8 gotchas (untracked component bodies, disposal on re-render, deprecated 0.7 names). Use when writing or reviewing any Leptos view/component/reactive code.
---

# Leptos 0.8 (pinned: 0.8.20) — CSR reference

This repo is **CSR-only** (wasm, `mount_to_body`). All semantics below verified against leptos 0.8.20 / leptos_macro 0.8.17 / reactive_graph 0.2.14 / tachys 0.2.18 local sources; API names cross-checked with the 0.8 book (book.leptos.dev).

## The most important rules (both burned this repo)

### 1. Component bodies run once per invocation, untracked — and their state dies on re-run

`leptos_macro-0.8.17` wraps every `#[component]` body in `untrack_with_diagnostics` (component.rs:350-359): the body runs with **no reactive observer** — top-level signal reads do not subscribe and never re-run it. The wrapper is re-invoked only when the **parent's dynamic slot** (`{move || …}` in the parent's view!) re-runs, and that re-run goes through `Owner::with_cleanup` (render_effect.rs:263-266), which **disposes every signal/memo/effect/StoredValue the previous run created** (owner.rs:303-310).

Consequences:

- **Never pick UI from a top-level `if signal.get()` in a body** — it is evaluated once and never flips. Put the switch in a tracked closure (`view! { {move || if …} }`) or `<Show when=…>`, or drive it through leptos_router — see the `leptos-router` skill. The repo's countdown→scoreboard switch (in `animeitor-client/src/views/sedes.rs`) uses a memoized branch inside the contest route's view: `Memo::new(move |_| !timer_negative)` + `{move || if memo.get() { board } else { countdown }}` — the memo flips only on zero-crossing, not per tick. Do NOT use `ProtectedRoute` for it: its guard closures run in a context-less scope (see the leptos-router skill).
- State that must survive re-renders lives **outside** the body: a parent's `StoredValue`, context, or a `static`/`OnceLock` (the timer signal in `api.rs` uses a `OnceLock`).
- **Signals cached in a `static`/`OnceLock` must be created under a detached `Owner::new_root(None)`** (then `std::mem::forget` the owner): an arena-allocated signal is disposed when its owner is disposed, and writing to it afterwards panics. A cache created lazily inside a transient route-render closure (like the per-event timer in `api.rs::create_timer`) would die with that closure's owner.
- One-shot setup in a body runs once per *invocation*, not once per mount — guard with `OnceLock` if the component can re-invoke.

### 2. In `view!`, `{closure}` is reactive; `{expression}` is static — and closures are moved

- `{move || value.get()}` — the closure is wrapped in a `RenderEffect` and **invoked repeatedly** when its tracked reads change (leptos_macro view/mod.rs:711-731 → tachys reactive_graph/mod.rs:42-52). Must be `FnMut + Send + 'static`.
- `{value.get()}` or any non-closure expression — evaluated **once**, rendered statically.
- `{signal}` shorthand is reactive (signals implement `Render` directly).
- A closure **variable** used as `{my_closure}` is moved by value into the render (one move per build). If `my_closure` was itself captured by-value into an outer re-runnable closure, the outer closure becomes `FnOnce` — the fix that works in this repo (sedes.rs `board`): create the inner closures **fresh inside the outer closure on each call**.
- Calling a closure once and rendering the result (`{my_closure()}`) **freezes** its reactivity — only do that when the result is meant to be static.
- `//` comments inside `view!` are fine (used in `views/placement.rs`).
- `match`/`if` inside the template: write them as `{match …}` blocks with `.into_any()` per branch (branches must share one type). `Option<V>` renders nothing for `None`.
- Iteration: plain `{move || list…}` re-creates nodes per change; `<For each=… key=|item| … children=…>` does keyed diffing — for this repo's ~1500 team rows use the per-team signal pattern from CLAUDE.md, never a whole-list re-render.
- A component body ending in `view! { … };` (trailing semicolon) is a deliberate compile error.

## Signals and reactive primitives

```rust
let (count, set_count) = signal(0);          // ReadSignal + WriteSignal; both Copy
let value = RwSignal::new(0);                // read/write in one handle, Copy
let double = Memo::new(move |prev| count.get() * 2); // lazy; closure gets Option<&T> prev
let derived = move || count.get() * 2;       // derived-signal closure
```

- Reading (all tracked): `.get()` (clones), `.with(|v| …)` (by reference), `.read()` (guard; panics if written while held). Writing: `.set(v)`, `.update(|v| …)`, `.write()` (guard). Prefer `.with`/`.update` for `Vec`/maps.
- **Memos are lazy** — they don't run until first read, and recompute only when a tracked source changed. `Signal::derive(…)` recomputes on every read (no caching) — pick per need.
- `Effect::new(move |prev| { … })` — **async, runs on the next tick** (not synchronously); the callback is `FnMut(Option<T>) -> T`, returning the next `prev`. `Effect::watch(deps_fn, handler, immediate)` for dependency-tracking effects. Don't write to signals read by the same effect.
- `Signal<T>` (via `#[prop(into)]` or `Signal::derive(...)`) accepts `ReadSignal`, `Memo`, `RwSignal`, or a derived closure — the idiomatic prop type for changing values. `Signal::stored(v)` for constant-backed signals.
- `StoredValue::new(v)` — non-reactive `Copy` handle; `.get_value()/.set_value()`. Arena-allocated under the current owner: **once per owner** — re-created when the body re-runs.
- `provide_context(v)` / `use_context::<T>() -> Option<T>` — scoped DI, walks the owner tree.
- `untrack(|| …)` — read without subscribing. `batch(|| …)` — defer effect notifications.
- Signals require `Send + Sync` (the graph is multi-threaded): `web_sys`/browser types need the `*_local` variants — `signal_local`, `RwSignal::new_local`, `StoredValue::new_local`, `LocalResource`. Accessing a local-storage signal cross-thread panics.
- `on_cleanup(f)` inside an effect body runs before the next effect run and on disposal.

### Deprecated 0.7 names (0.8.20 emits warnings — don't use)

`create_signal` → `signal` · `create_rw_signal` → `RwSignal::new` · `create_effect` → `Effect::new` · free `watch` → `Effect::watch` · `create_trigger` → `ArcTrigger::new`. There is **no** `create_memo` free fn, and **no** `#[signal]`/`#[memo]`/`#[effect]` attribute macros in 0.8 (those are 0.9).

## Components and props

```rust
#[component]
pub fn Progress(#[prop(into)] progress: Signal<i32>, #[prop(default = 100)] max: u16) -> impl IntoView {
    view! { <progress max=max value=progress /> }
}
// usage:
<Progress progress=count />
<Progress progress=Signal::derive(double_count) />
```

- `#[prop(into)]` — `.into()` on the passed value. `#[prop(optional)]` — `Default::default()` when omitted. `#[prop(default = expr)]` — custom default. `#[prop(attrs)]` — collect extra attributes.
- Plain (non-signal) props are frozen at build time — props that change over time MUST be signal types.
- `children: Children` (`Box<dyn FnOnce() -> AnyView + Send>`) for `{children}` slots; `TypedChildren<Chil>` for typed slots.
- Generic props: `impl Fn() -> T + Send + Sync + 'static`; specify types at the call site as `<Component<Type>/>`. Optional generic props fail inference — use a concrete type.
- Return types: `impl IntoView` is the convention; `-> AnyView` + `.into_any()` when branches differ; `()` and other renderable types are also legal (`IntoView` is a blanket impl).
- `#[component(transparent)]` — no wrapper DOM node.

## Rendering control (CSR signatures)

- `<Show when=move || cond.get() fallback=…>` — renders each branch once, keeps it mounted (unlike `{move || if …}` which re-creates per change).
- `<Suspense fallback=…>` — catches pending resources; `<Transition fallback=…>` — keeps showing previous children while pending.
- `Suspend::new(async { … })` — suspends until the future resolves (must be under Suspense/Transition).
- `<ErrorBoundary fallback=move |errors| …>` — errors from `Result` values rendered in children.
- `<For each=… key=|item| … children=…>` — keyed iteration.
- `<Await future=…>{|value| …}</Await>` / `<ShowLet some=…>{let:value …}`.
- `LocalResource::new(|| async { … })` — `Copy`; one-shot cached resource (`.await` suspends; **waits one executor tick before resolving** on the client); `Resource::new(source, fetcher)` for source-driven refetch.
- `spawn_local(async { … })` — unscoped wasm task (`leptos::task::spawn_local`). `leptos::task::spawn` runs the future under the current owner + observer.

## Mounting (main.rs)

```rust
leptos::task::spawn_local(async move {
    let config = client_sdk::SdkConfig::load().await;   // runtime config BEFORE mounting
    animeitor_client::init_config(config);
    mount_to_body(|| { provide_global_settings(); view! { <Sedes /> } });
});
```

`mount_to_body`'s closure runs in the **root owner**, which lives for the whole app — the right place for signals/state that must outlive any component body re-run.
