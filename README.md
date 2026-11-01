

# CVGB

A Game Boy Color (CGB) emulator written in Rust

## Status

Can run Tetris. Currently working on making more games work.

Audio support is really janky.

## Test results

<!-- TESTS:START -->
| Suite | Passing | Total | % |
|---|---:|---:|---:|
| age_test_roms | 0 | 29 | 0% |
| blargg | 31 | 40 | 78% |
| cvgb-core | 2006 | 2006 | 100% |
| gbmicrotest | 95 | 513 | 19% |
| mooneye_test_suite | 46 | 83 | 55% |
| All | 2178 | 2671 | 82% |
<!-- TESTS:END -->

## Run

```sh
cargo run --release
```

Has only been tested on Linux. `nix develop` provides all dependencies.

| Key | Action |
|---|---|
| Arrows | D-pad |
| X / Z | A / B |
| S / A | Start / Select |
| F3 | Options and debug menu |

## Test

```sh
./scripts/fetch_test_roms.sh
cargo nextest run --release
```
