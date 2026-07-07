# Test fixtures

Static JSON samples used by Rust unit tests under `src/`. Paths are resolved relative to
the repository root via `CARGO_MANIFEST_DIR`.

| File | Used by | Purpose |
|---|---|---|
| `dataset_labels_string.json` | `datasets::labels_tests` | String-array label list |
| `dataset_labels_wrapped.json` | `datasets::labels_tests` | Wrapped `{ "data": [...] }` label response |
| `benchmark_verification_files.json` | `benchmarks::save_tests` | Required file list from verification API |
| `benchmark_verification_response.json` | `benchmarks::save_tests` | Successful verification verdict JSON |

Add new fixtures here when extending mock HTTP or parser coverage.
