# Fuzzing

Targets (thin wrappers over the codec `parse`; contract: never panic):

| target | codec |
|---|---|
| `fuzz_dbml_parse` | `dibibest_core::dbml::DbmlCodec` |
| `fuzz_sql_parse` | `dibibest_core::sql::SqlCodec` |

Needs nightly + `cargo-fuzz`:

```sh
rustup toolchain install nightly
cargo install cargo-fuzz --locked

cargo +nightly fuzz list
cargo +nightly fuzz run fuzz_dbml_parse                        # until Ctrl-C
cargo +nightly fuzz run fuzz_sql_parse -- -max_total_time=600  # 10-minute smoke
```

## M4 exit gate (not run in CI)

A **1 hour clean run per target** — attach the logs to the release:

```sh
cargo +nightly fuzz run fuzz_dbml_parse -- -max_total_time=3600 2>&1 | tee fuzz_dbml_parse.log
cargo +nightly fuzz run fuzz_sql_parse  -- -max_total_time=3600 2>&1 | tee fuzz_sql_parse.log
```

Clean = exit 0, no files in `fuzz/artifacts/`. The nightly
`.github/workflows/fuzz.yml` only runs the 10-minute smoke.

## Reproducing a crash

```sh
cargo +nightly fuzz run fuzz_dbml_parse fuzz/artifacts/fuzz_dbml_parse/crash-<hash>
```

Turn the input into a regression test in `crates/dibibest-core/tests/` before fixing.
