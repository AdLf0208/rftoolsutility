# Working on a native Horizon mod (SDK format)

> **Build slots (v2, 2026-11-09):** every build/test/e2e goes through the
> campaign runner (`ports/tools/build-slot.sh` from the campaign root,
> `../tools/build-slot.sh` from this port). It **blocks in the kernel** until a
> slot is free — a freed slot is taken in ~0.07 s — so **never poll**: no
> `while ! slot; sleep` loop, no background build. It sets `CARGO_BUILD_JOBS` per
> step (build/test 3, stage 2, e2e 1), journals grants/releases, and a timeout
> exits **75**. Read `../_shared/build-slots-usage.md` before running anything;
> observe with `status` / `watch` / `events` / `stats`, never with `ps`+`sleep`.
> Never pass `--target-dir`, never run a bare `cargo` outside a lease.

**Format: the flat SDK package (2026-11-18).** `.hnmod`, `mixins/<id>/`,
`horizon-mixin.toml`, `compatibility.json` and the staging tools
(`horizon.py`/`package.py`) are retired; the campaign keeps them only as evidence
under `../_shared/evidence/<id>/retired-hnmod-0.1.0/`. This port is one `.hmod`:

- `horizon-mod.toml` — the only manifest and the only identity (`api = "0.2"`,
  `[native]` pinned to the exact `game`/`horizon`/`mixins` commits);
- `Cargo.toml` — one crate whose only dependency is `horizon-mod-api` from
  `.horizon/sdk/` (fetched by `cargo horizon-mod`, never packed);
- `src/lib.rs` — the one `#[mod_api::horizon_mod] impl Hooks`, the Parts and the
  game glue; `src/runtime.rs` — the pure logic, **byte-identical** to the
  retired bundle's copy, with its own tests;
- `assets/` and `data/` — the carried content, at the package root.

Read `README.md` and `notes.md` before editing. Keep this directory separate from
Horizon and horizon-mixins; never modify the pin checkouts or player resources.
Do not commit binaries, `.horizon/`, `target/` or a generated `Cargo.lock` (a
packed lock changes the artifact; see `notes.md`).

Gameplay logic belongs in `src/runtime.rs` with a test; `src/lib.rs` only
translates between the SDK's types and the runtime's. A hook is a typed `Hooks`
method or a Part attached through the `Registrar` — never a free function, never
an unstamped name.

Validate, in one lease each:

```sh
BUILD_LOCAL=1 ../tools/build-slot.sh run --job sdk-rfu --port ports/rftoolsutility --step test \
    -- bash -c 'cd ports/rftoolsutility && CARGO_INCREMENTAL=0 cargo test --offline'
BUILD_LOCAL=1 ../tools/build-slot.sh run --job sdk-rfu --port ports/rftoolsutility --step lint \
    -- bash -c 'cd ports/rftoolsutility && cargo fmt --check &&
        CARGO_INCREMENTAL=0 cargo clippy --all-targets --offline -- -D warnings'
BUILD_LOCAL=1 ../tools/build-slot.sh run --job sdk-rfu --port ports/rftoolsutility --step build \
    -- bash -c 'cd ports/rftoolsutility && rm -f Cargo.lock && CARGO_INCREMENTAL=0 cargo horizon-mod build'
BUILD_LOCAL=1 ../tools/build-slot.sh run --job sdk-rfu --port ports/rftoolsutility --step check \
    -- bash -c 'cd ports/rftoolsutility && cargo horizon-mod check rftoolsutility.hmod &&
        cargo horizon-mod fits rftoolsutility.hmod --build nightly'
```

`cargo horizon-mod build` packs; it does not compile against the game. The real
native build is `horizon-launcher native-build <port> --instance <i>
--trust-native-code`, which is not installed on the campaign box — so the
compile evidence is clippy/`cargo test` against the pinned SDK plus `check`/
`fits`, and the in-game leg (`tools/e2e.sh`) runs wherever a launcher-built game
exists. Keep validation claims and supported revisions honest.
