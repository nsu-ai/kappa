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

result = bm.save_benchmark(predictions, metrics, model_path="./model")
response = bm.submit_benchmark(upload_artifacts=True)   # inference + weights + link
```

`Benchmarks` is returned by `load_benchmark()` — it is **not** exported as `kappa_apk.Benchmarks`.

---

## Benchmarks methods

| Method | Description |
|---|---|
| `benchmark_id` | UUID property |
| `dataset(dataset_path?)` | Download the evaluation set → `list[DatasetItem]`; cache under `~/cache/kappa-framework/benchmarks/{id}/` |
| `setup_project()` | Sample fingerprints from nearest `src/` directory |
| `set_model_id(model_id)` / `get_model_id()` | Override model ID for submission |
| `debug_benchmark_details()` | Human-readable status string |
| `save_benchmark(predictions, metrics?, model_path?)` | Build `BenchmarkResult` in memory |
| `saved_result` | The last saved `BenchmarkResult`, or `None` |
| `submit_benchmark(strict?, model_version_id?, complete_inference?, upload_artifacts?, artifact_paths?, on_progress?)` | `POST …/models/inferences/{model_id}`, upload artifacts, then link it to the benchmark |

`save_benchmark` accepts predictions-only results, or attaches model/application metadata from `setup_project()`, `model_path`, or prior `set_model_path`.

Saving an inference alone leaves the benchmark at *Pending Inference*. `submit_benchmark()` therefore also calls `POST …/benchmarks/inferences/{benchmark_id}/{model_version_id}`, which advances it to *Inference Completed*. The version comes from `model_version_id` or the benchmark's `mlmodelVersionId`; pass `complete_inference=False` to submit without linking.

`save_benchmark(model_path=…)` only records file *metadata* (name, size, hash). To store the weights themselves, add `upload_artifacts=True`:

```python
bm.save_benchmark(predictions, metrics, model_path="./model")
bm.submit_benchmark(
    upload_artifacts=True,                     # uploads every file under ./model
    on_progress=lambda name, sent, total, pct: print(f"{name} {pct}%"),
)
```

Artifacts default to `file_category=3` (Model); files past the server's sync cap take a resumable multipart session, so multi-GB checkpoints work. Pass `artifact_paths=[…]` to upload something other than the saved `model_path`. For a plain (non-benchmark) run, `client.write_model_inference()` does the same thing in one call — see [KappaApkClient.md](KappaApkClient.md#model-artifacts).

---

## Evaluation set download (Kappa ≥ 2.11.0)

`bm.dataset()` and `client.download_benchmark_dataset_package()` follow the same order as the web UI:

1. **Dataset package** — `GET /data-micro-services/v2/datasets/versions/{datasetId}/{versionNo}/package`, using the `datasetId` / `datasetVersionNo` from the benchmark detail. Multi-shard manifests are fetched shard by shard.
2. **Benchmark proxy** — on `401` / `403` / `404` (you hold `benchmark.read` but not `dataset.read`), retry via `GET /model-micro-services/v2/benchmarks/datasets/{benchmark_id}/package` plus `…/package/shards/{name}`.
3. **Legacy zip** — either path uses its single-zip route when the manifest reports `legacySingleZip` or lists at most one shard.

Large 2.11 versions no longer have a single zip, so calling the legacy benchmark download route directly returns `404`.

| Situation | What you get |
|---|---|
| Version archive still building | `RuntimeError` — wait for the version build job, then retry |
| Download approval pending (`publish_type = 3`) | `PermissionError` |
| No `benchmark.read` either | Error from the proxy call |

```python
path = client.download_benchmark_dataset_package("eaa50325-…")   # returns cache dir
manifest = client.get_benchmark_dataset_package_manifest("eaa50325-…")
```

---

## Benchmark management

| Method | Endpoint |
|---|---|
| `filter_benchmarks(...)` / `list_benchmarks()` | `GET /benchmarks` — `benchmark_id`, `model_id`, `dataset_id`, `benchmark_status`, `user_id`, date range, `order_by` / `order`, `page` / `size` |
| `get_benchmark(id)` | `GET /benchmarks/{id}` — includes `datasetVersionNo` and `benchmarkStatus` |
| `create_benchmark(body)` / `update_benchmark(id, body)` / `delete_benchmark(id)` | `POST` / `PUT` / `DELETE /benchmarks[/{id}]` |
| `complete_benchmark_inference(id, model_version_id)` | `POST /benchmarks/inferences/{id}/{model_version_id}` |
| `list_benchmark_remarks(id)` / `add_benchmark_remark(id, msg)` | `GET` / `POST /benchmarks/{id}/remarks` (1–4000 chars) |
| `get_benchmark_flow_schema()` | `GET /benchmarks/flow-schema` |

### Expert & dataset selection

| Method | Endpoint |
|---|---|
| `respond_to_benchmark_expert_request(id, accept)` | `POST /benchmarks/management/expert/{id}/{2 or 0}` |
| `assign_benchmark_expert(id, expert_id)` | `POST /benchmarks/management/{id}/{expert_id}` — needs `benchmark.manage` |
| `propose_benchmark_dataset(id, dataset_id, version_id)` | `POST /benchmarks/management/expert/{id}/{dataset_id}/{version_id}` |
| `confirm_benchmark_dataset(id)` / `reject_benchmark_dataset(id)` | `POST …/dataset/confirm` / `…/dataset/reject-proposal` |
| `get_benchmark_dataset_attachments(dataset_id, version_id)` | `GET /benchmarks/datasets/attachments/{dataset_id}/{version_id}` |

### Review & report

| Method | Endpoint |
|---|---|
| `get_benchmark_review(id, expert_id)` | `GET /benchmarks/review/{expert_id}/{id}` — first call moves status 5 → 6 |
| `save_benchmark_review(id, expert_id, review)` | `PUT` same path |
| `finalize_benchmark_review(id, expert_id)` | `POST` same path — schedules report generation |
| `regenerate_benchmark_report(id)` | `PATCH /benchmarks/review/report/{id}` — needs `benchmark.manage` |
| `get_benchmark_report(id)` | `GET /benchmarks/report/{id}` |
| `download_benchmark_report(id, dest_path, lang?)` | `GET /benchmarks/report/download/{id}/{lang}` — `"en"` (default) or `"ru"` |

```python
review = client.get_benchmark_review(benchmark_id, expert_id)
client.save_benchmark_review(benchmark_id, expert_id, {
    "reviews": {"<entity-uuid>": {"entityId": "<entity-uuid>", "score": 4, "remark": "ok"}},
    "finalScore": 4,
})
client.finalize_benchmark_review(benchmark_id, expert_id)
client.download_benchmark_report(benchmark_id, "report.pdf", lang="ru")
```

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
| Benchmark evaluation set | `~/cache/kappa-framework/benchmarks/{benchmark_id}/` |
| Model run files | From `model_path` or `setup_project()` metadata |

A download is only cached once every shard has been extracted, so an interrupted download is retried from scratch rather than left half-complete.

[← Index](README.md)
