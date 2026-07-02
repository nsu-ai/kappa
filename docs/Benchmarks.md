# Benchmarks

**Русская версия:** [ru/Benchmarks.md](ru/Benchmarks.md)

---

## Workflow overview

```python
bm = client.load_benchmark("eaa50325-5f3d-4e66-b7b5-b18e5a587563")

data = bm.dataset()              # download + cache benchmark dataset
bm.setup_project()               # optional: attach src/ file fingerprints

predictions = [
    {"entity_id": item.entity_id, "original": ..., "predicted": ...}
    for item in data
]
metrics = {"accuracy": 0.95, "f1": 0.92}

result = bm.save_benchmark(predictions, metrics)
response = bm.submit_benchmark()   # POST inference result to model service
```

`Benchmarks` is returned by `load_benchmark()` — it is **not** exported as `kappa_apk.Benchmarks`.

---

## Benchmarks methods

| Method | Description |
|---|---|
| `benchmark_id` | UUID property |
| `dataset(dataset_path?)` | Download archive → `list[DatasetItem]`; cache under `~/cache/kappa-framework/benchmarks/{id}/` |
| `setup_project()` | Sample fingerprints from nearest `src/` directory |
| `set_model_id(model_id)` / `get_model_id()` | Override model ID for submission |
| `debug_benchmark_details()` | Human-readable status string |
| `save_benchmark(predictions, metrics?, model_path?)` | Build `BenchmarkResult` in memory |
| `submit_benchmark()` | `POST /model-micro-services/v2/models/inferences/{model_id}` |

`save_benchmark` accepts predictions-only results, or attaches model/application metadata from `setup_project()`, `model_path`, or prior `set_model_path`.

---

## BenchmarkVerification

Standalone helper to verify application files against server requirements (public verification endpoints — no JWT required).

```python
from kappa_apk import BenchmarkVerification

bv = BenchmarkVerification(
    server_url="https://kappa.nsu.ru:8061/",
    benchmark_id="8c97da09-375c-47cd-814c-d1798dbd48f4",
    path="/path/to/project",
)
ok = bv.result()   # bool — scans dir, posts hashes to v2 verification API
```

HTTP (via gateway):

- `GET …/model-micro-services/v2/benchmarks/app/verifications/files/{benchmark_id}`
- `POST …/model-micro-services/v2/benchmarks/app/verifications/{benchmark_id}`

Missing `valid` / `success` in the response JSON is treated as **failure** (`False`).

---

## Caching

| Resource | Cache path |
|---|---|
| Benchmark dataset | `~/cache/kappa-framework/benchmarks/{benchmark_id}/` |
| Model run files | From `model_path` or `setup_project()` metadata |

[← Index](README.md)
