# (history) RFTools Utility slice-1 plan — the retired `.hnmod` format

> This plan describes the 0.1.0 bundle, which Horizon retired on 2026-11-18 (`.hnmod` →
> flat SDK `.hmod`). The current layout, hook mapping and ladder are in `notes.md`
> § "Migration to the SDK format"; the `.hnmod` tree and packaging tools are kept under
> `../_shared/evidence/rftoolsutility/retired-hnmod-0.1.0/`.

The brief: one machine whose loop closes without a GUI, with the jar's numbers,
a `/rftoolsutility info` readout, and a control that an empty/blocked machine
does nothing. Approach from `horizon/docs/modpacks.md` (`machines`): a block
entity, a fluid tank, a server tick, and carried content — not a data-only
block.

## Deliverable

`ports/rftoolsutility/rftoolsutility.hnmod` — one file carrying the code
(`mixins/rftoolsutility/`) **and** the content
(`mixins/rftoolsutility/data/rftoolsutility-content.hmod`, the AE2 pattern),
with `LICENSE` + `NOTICE.md` inside the archive.

## Files

| File | Role |
|---|---|
| `mixins/rftoolsutility/runtime.rs` | pure: capacity, bucket map, fill level, insert/extract, the bucket-click decision, the info line's numbers. Unit tests first. |
| `mixins/rftoolsutility/support.rs` | glue: the block entity, the fluid table, the tick, the click, the commands |
| `mixins/rftoolsutility/{block_entity,block_entity_ticks,block_entity_fluids,block_entity_tick,right_click_block,register_commands,perform_command}.rs` | one private helper per injection |
| `mixins/rftoolsutility/embedded_content.rs` | writes the embedded `.hmod` out and appends it to the dev list |
| `content/` | the carried tank: blockstate (9 levels), 9 block models + the inventory model, item model, 9 textures + 3 base textures, lang, block/item/creative-tab declarations, loot table, recipe, advancement, tags |
| `tools/assemble_content.py` | builds `content/` from the jar, deterministically |
| `tools/package.py` | builds the `.hnmod` |
| `tools/e2e.sh` | the in-game scenario, `EMBEDDED=1` / `CONTROL=1` / `SELFTEST=1` |
| `notes.md`, `FINDINGS.md`, `VALIDATION.md`, `CONTENT.md`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md` | the record |

## Tasks

1. **Runtime, red first.** `cargo test` with `todo!()` bodies fails; then the real
   bodies pass. Tests pin: the capacity 32,000 and a bucket 1,000; the level
   curve 0/1/2/4/8 at 0/1/4,000/12,000/32,000 mB; one fluid at a time; a fill
   clamps at the capacity and reports the leftover; a drain gives whole buckets
   and refuses a partial one; a water bucket fills, an empty bucket drains, a
   lava bucket into a water tank is refused, an empty tank with an empty bucket
   does nothing, a non-bucket is not the tank's click.
2. **Content.** `tools/assemble_content.py` → `content/`, `cargo horizon-mod
   build` → `content/rftoolsutility.hmod`, notices appended.
3. **Glue.** The seven helpers + `support.rs`; `stage`/`check` clean.
4. **Ladder.** fmt, clippy, stage, check, embedded tests, fast build — in build
   slots.
5. **Package.** `tools/package.py` → `ports/rftoolsutility/rftoolsutility.hnmod`.
6. **e2e, fail-capable.** The End arena: a tank placed from its item is empty
   (`level=0`); a water bucket right-click fills exactly 1,000 mB and swaps the
   hand to an empty bucket; the block state's level rises; an empty bucket
   drains exactly 1,000 mB back and gives a water bucket; `/rftoolsutility tank`
   fills to 32,000 and one more bucket changes nothing (capacity); a lava bucket
   into a water tank does nothing; an empty tank with an empty bucket does
   nothing; a vanilla cauldron takes the same bucket (differential); a
   non-bucket click does nothing. `EMBEDDED=1` runs with no `--dev-mod` plus an
   unusable-`.horizon-cache` control; `SELFTEST=1` flips an assertion and exits
   non-zero with a `FAIL:` verdict.
7. **Gates + record.** `check-e2e-logs.py --strict`, `check-skills.py --notes`,
   `upstream_ready.py`; `notes.md` with class/tier, licence, slice boundary,
   gaps, defects, the `Skills:` line.
