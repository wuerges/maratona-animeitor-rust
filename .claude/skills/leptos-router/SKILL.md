---
name: leptos-router
description: Reference for leptos_router 0.8 (pinned 0.8.15) as used in this repo's CSR client — Router/Routes/Route/ProtectedRoute/Redirect/A components, the path! macro, navigation hooks, and confirmed gotchas. Use when adding or changing routes, navigation, redirects, URL params/queries in client-v2.
---

# leptos_router 0.8 (pinned: 0.8.15) — CSR reference

One `Router` per app, wrapping one `Routes` with `Route` children and a `fallback` shown when nothing matches. Verified against the 0.8.15 source; all semantics below are from that version.

## Minimal setup (CSR)

```rust
use leptos_router::components::{ProtectedRoute, Route, Router, Routes};
use leptos_router::{path, NavigateOptions};

view! {
    <Router>                                          // base prop optional, defaults to ""
        <Routes fallback=move || view! { <Landing /> }>
            <Route path=path!("/") view=Home />       // view: Fn() -> View + Send + Clone + 'static
            <Route path=path!("/users/:id") view=move || view! { <User /> } />
        </Routes>
    </Router>
}
```

- `Router` is `#[component(transparent)]` — no DOM wrapper. It reads `window.location` (CSR), provides the router context, and intercepts `<a>` clicks globally.
- `Routes` panics outside a `Router`. `{()}` as children = "always fallback" router (used in `sedes.rs` for the landing-only case).
- A route matches only when the **entire** path is consumed (with `""`/`"/"` tolerated as the rest). Matching is case-sensitive and first-match-wins among siblings.

## The `path!` macro

Single string literal per call: `"foo"` static, `":name"` param, `":name?"` optional param, `"*rest"` wildcard (must be last). Example: `path!("/animeitor/:event/:contest")`. Segment names may only contain `[A-Za-z0-9-._~@]`; `//` is a compile error.

## Components

| Component | Props | Notes |
|---|---|---|
| `Router` | `base` (default `""`), `set_is_routing`, children | Transparent; provides router context |
| `Routes` | `fallback` (FnOnce + Clone + Send), `transition`, children | Re-matches when the URL *path* changes — query-only changes do NOT rebuild |
| `Route` | `path`, `view`, `ssr` | No children prop; view must be `Send + Clone + 'static` |
| `ParentRoute` | like `Route` + children | Nesting |
| `ProtectedRoute` | `path`, `view`, `condition`, `redirect_path`, `fallback`, `ssr` | Guard: see below |
| `Redirect` | `path`, `options` | Navigates on render; **push by default** (`replace: false`) |
| `A` | `href` (also `Fn() -> String`), `target`, `exact`, `strict_trailing_slash`, `scroll`, children | There is **no `Link`** in 0.8.15 — use `A` |
| `Form` | `method`, `action`, `enctype`, `replace`, … | GET forms navigate client-side; POST via gloo_net |
| `Outlet` | — | Renders the nested child route's view |

### `ProtectedRoute` (route guards)

```rust
<ProtectedRoute
    path=path!("/animeitor/:event/:contest")
    view=board                                  // Fn() -> View + Send + Clone + 'static
    condition=move || Some(!timer.with(|pair| pair.is_negative()))  // Fn() -> Option<bool>; None = loading
    redirect_path=move || countdown_path.clone()                    // Fn() -> P, P: Display
    fallback=|| view! { <p>"…"</p> }            // shown while condition is None
/>
```

- `condition` **is reactive**: it is evaluated inside a `Transition` render closure, so it re-runs when the signals it reads change. `Some(true)` renders the view, `Some(false)` renders a `<Redirect>` to `redirect_path()`, `None` renders the fallback.
- `Redirect` **pushes** a history entry by default. A guard that redirects to a route which itself redirects back loops. This repo redirects to a *different* route whose view navigates back with `replace: true` — see `sedes.rs` (countdown switch).

## Hooks (`leptos_router::hooks`)

| Hook | Returns | Where it works |
|---|---|---|
| `use_navigate()` | `impl Fn(&str, NavigateOptions) + Clone` | Anywhere under `Router` — including route view closures and the fallback |
| `use_location()` | `Location { pathname, search, query, hash, state }` (memos/signals) | Under `Router` |
| `use_params_map()` | `Memo<ParamsMap>` (`.get("id") -> Option<String>`) | **Only inside a matched route's view** (or its descendants) — panics in the fallback |
| `use_params::<T>()` | `Memo<Result<T, ParamsError>>` — `T: Params` derive | Same as above |
| `use_query_map()` / `use_query::<T>()` | Memo over the URL query | Under `Router`, any scope |
| `use_matched()` | `Memo<String>` — matched path | Inside a matched route |

`NavigateOptions` (defaults): `resolve: true`, `replace: false`, `scroll: true`, `state: None`.

`#[derive(Params)]` (from `leptos_router::params::Params`): maps query/param keys onto `Option<T: FromStr>` fields (stable Rust: only `Option<T>`).

## Confirmed gotchas (0.8.15 source)

1. **Route `view` closures run ONCE per match** — never re-executed on signal changes (same-route param changes update params/URL signals without re-running the view; query-only changes don't rebuild at all). All reactivity must live *inside* the returned view: dynamic `{move || …}` children, `Effect`, `Memo`, signals.
2. **View/condition/redirect closures must be `Clone + Send`** — capture `ReadSignal`s (Copy) or `Arc`s; clone `String`s inside (`move || path.clone()`).
3. **`use_params`/`use_params_map` panic outside a matched route** ("Tried to access params outside the context of a matched <Route>.") — never call them in the `fallback`.
4. **`Redirect` pushes** by default — for guard redirects prefer landing on a different route that navigates back with `replace: true` (see the countdown pattern in `client-v2/src/views/sedes.rs`).
5. **Matching is exact and case-sensitive**; a trailing slash breaks matching in practice — always link to the canonical path without a trailing slash (the landing in `client-v2/src/views/landing.rs` does this).
6. **Query-only navigation does not re-render the fallback or re-run route views** — read `use_query` in a memo/signal if the query drives UI.
7. Navigation is signal-first: `current_url` updates before browser history; the address bar can lag when route views suspend.
8. **A component that renders the `<Router>` itself runs OUTSIDE the router context** — calling `use_query`/`use_navigate` in its body panics ("unreachable" in the wasm console). Keep the Router-wrapping component's body free of router hooks; put them in the route views (components rendered *inside* `<Routes>`).
9. **One Router per app — never branch at mount time** into two Routers (e.g. landing vs contest): the mounted router never switches, so navigation changes the URL but not the page. `client-v2/src/views/sedes.rs` is the canonical pattern: a single `<Router>` whose routes are the landing (`path!("/")`), the contest (`ProtectedRoute path!("/animeitor/:event/:contest")`) and the countdown, with the event/contest read via `use_params::<ContestParams>()` inside the route views — reactive, so contest→contest navigation rebuilds the screen through the `{move || …}` closure instead of relying on view re-runs (which don't happen on same-route param changes).
