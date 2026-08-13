# KappaApkClient

**Русская версия:** [ru/KappaApkClient.md](ru/KappaApkClient.md)

Main entry point for the SDK. Wraps authenticated HTTP to Kappa-framework **v2** microservices via the Traefik gateway.

```python
from kappa_apk import KappaApkClient

client = KappaApkClient(base_url: str, login_id: str, passwd: str)
```

Paths below are appended to `base_url`. All authenticated dataset/benchmark calls send `Authorization: Bearer <token>`.

---

## Session & client

| Method | HTTP | Description |
|---|---|---|
| `connect()` | `POST /user-micro-services/v2/session/new` | Login; returns user info dict + token |
| `close()` | `DELETE /user-micro-services/v2/session` | Logout; invalidate token (Bearer only) |
| `get_token()` | — | Current bearer token or `None` |
| `is_authenticated()` | — | Whether a token is stored |
| `get_base_url()` / `set_base_url(url)` | — | Gateway URL get/set |
| `__enter__` / `__exit__` | — | Context manager: connect on enter, close on exit |
| `make_request(method, endpoint, data?, token?)` | *custom* | Low-level authenticated request |

---

## Dataset listing & lookup

| Method | HTTP | Notes |
|---|---|---|
| `list_datasets(page?, size?, order_by?, order_keyword?)` | `GET /data-micro-services/v2/datasets/filter` | Raw JSON page (defaults: page=1, size=200) |
| `list_datasets_typed(...)` | same | Returns `list[Dataset]` |
| `filter_datasets(search?, dataset_tags?, dataset_type?, dataset_status?, publish_type?, page?, size?, order_by?, order_keyword?, query_all?, start_date?, end_date?, selected_version_id?, selected_version_no?)` | `GET …/datasets/filter` | Rich filters (defaults: page=1, size=20) |
| `get_dataset_details(dataset_id?, dataset_name?)` | `GET …/datasets/filter` | Single `Dataset` |
| `get_dataset_fields(dataset_id)` | `GET …/datasets/fields/{id}` | Input field schema |

`list_datasets` and `filter_datasets` hit the same filter endpoint with **different default pagination**. Pages are 1-based and the envelope is `{items, total, page, size, pages}`.

- `query_all=False` narrows results to datasets you own, are assigned to, or that are shared with you. The default (`True`) also lists the public catalogue.
- `selected_version_id` / `selected_version_no` add `selectedVersionNo` and `selectedVersionBuildStatus` to each item, so you can check for `ready` before downloading.
- Sort direction is `order_keyword` here, but `order` on the entity filter.

---

| `get_system_config(tag)` | `GET /user-micro-services/v2/system/config/{tag}` | Config rows (e.g. `dataset_type`, `dataset_tags_1`) |
| `list_predefined_ml_tags(type_id)` | same (`dataset_tags_{type_id}`) | Display values for create forms |

Helpers (module-level): `join_ml_tags(primary, *extras)`, `ensure_primary_ml_tag_first(tags, primary)`, `validate_ml_tags(tags, predefined, require_primary_first=True)`.

---

## Dataset CRUD

| Method | HTTP |
|---|---|
| `add_dataset(dataset, check_tags=True)` | `POST …/datasets/new` |
| `update_dataset(dataset_id, update)` | `PUT …/datasets/{id}` |
| `delete_dataset(dataset_id, remark)` | `DELETE …/datasets/{id}` (soft-delete; recover via server `/datasets/recover`) |

`check_tags=True` (default) loads `dataset_tags_{dataset_type}` and requires the **first** tag to be predefined. Same rule for `create_model(..., check_tags=True)` with `mlModelType` / `mlModelTags`.

`dataset` / `update` accept typed models or plain `dict` (camelCase JSON keys).

---

## Labels

| Method | HTTP |
|---|---|
| `add_dataset_labels(dataset_id, labels)` | `POST …/datasets/labels/{id}` |
| `get_dataset_labels(dataset_id)` | `GET …/datasets/labels/{id}` → `list[DatasetLabel]` |
| `get_dataset_label_names(dataset_id)` | `GET …/datasets/labels/{id}` → `list[str]` |
| `update_dataset_label(dataset_id, label_id, label)` | `PUT …/datasets/labels/{id}` |

---

## Entities (samples)

| Method | HTTP |
|---|---|
| `add_dataset_entity(dataset_id, entity, file_paths?, version_id?)` | `POST …/datasetEntities/new/{id}` |
| `update_dataset_entity(dataset_id, entity_id, update, version_id?)` | `PUT …/datasetEntities/{id}/{entity_id}` |
| `list_dataset_entities(dataset_id, version_id?)` | `GET …/datasetEntities/{id}` |
| `get_dataset_entity(dataset_id, entity_id, version_id?)` | `GET …/datasetEntities/{id}/{entity_id}` |
| `filter_dataset_entities(dataset_id, entity_name?, entity_status?, version_id?, page?, size?, order_by?, order?, entity_id?, location_id?, assignment_filter?, start_date?, end_date?)` | `GET …/datasetEntities/filter/{id}` — 1-based pages; `assignment_filter` is `"assigned"` / `"not_assigned"` |
| `delete_dataset_entities(dataset_entity_ids, remark, version_id?)` | `DELETE …/datasetEntities` |

`file_paths` accepts file paths, a directory (immediate children), or `http(s)://` URLs.

---

## Versions & archives

| Method | HTTP |
|---|---|
| `get_dataset_version_details(dataset_id?, dataset_name?, version_id?, version_no?)` | `POST …/versions/details/list` or `GET …/versions/{id}/{ver}` |
| `download_dataset_version_archive(..., dataset_path?)` | `GET …/versions/archive/{id}/{ver}` |
| `download_dataset_version_package(...)` | `GET …/versions/{id}/{ver}/package` + shards |
| `get_dataset_version_package_manifest(dataset_id, version_no)` | `GET …/package` |
| `create_dataset_version(dataset_id, version)` | `POST …/versions/new/{id}` → may include `jobId` |
| `list_dataset_versions(dataset_id, version_availability?)` | `GET …/versions/{id}` |
| `delete_dataset_version(dataset_id, version_no)` | `DELETE …/versions/{id}/{ver}` |
| `publish_dataset_version(dataset_id, version_no, publish_type)` | `POST …/versions/publish/{id}/{ver}?publish_type=N` |
| `refresh_dataset_version(dataset_id, version_no)` | `PATCH …/versions/refresh/{id}/{ver}` |
| `get_version_build_job` / `wait_for_version_build_job` / `retry_version_build_job` | `…/versions/build-jobs/{jobId}` |

`publish_type`: `0` Not Published · `1` Private · `2` Open Source · `3` Public on Demand · `4` Purchase

Wait for `buildStatus=ready` (via `wait_for_version_build_job`) before publish/download on Kappa ≥ 2.11.

---

## Bulk mutations (Kappa ≥ 2.11)

| Method | HTTP |
|---|---|
| `mark_dataset_entities_labeled(dataset_id, dataset_entity_ids?, remark?, all_eligible?)` | `POST …/mark-labeled/{id}` |
| `bulk_self_verify_dataset_entities(dataset_id, status, comment?)` | `POST …/verification/self-verify-batch/{id}` |
| `auto_verify_dataset_entities(dataset_id)` | `POST …/verification/auto-verify/{id}` |
| `get_bulk_mutation_job` / `wait_for_bulk_mutation_job` / `cancel_bulk_mutation_job` | `…/bulk-mutation/jobs*` |
| `get_mark_labeled_stats` / `get_self_verify_stats` | stats GET endpoints |

Start endpoints return `jobId` — poll with `BulkMutationJob`. Delete/recover/delete-files are also async jobs.

---

## Training helpers

| Method | Description |
|---|---|
| `load_kappa_dataset(..., transform?, transform_input_mode?)` | Download (cached) + `KappaDataset` |
| `get_dataset_loader(..., loader_type?, batch_size?, shuffle?, drop_last?, transform?, tf_output_signature?)` | Framework-specific loader |

`loader_type`: `"kappa"` (default) · `"pytorch"` · `"transformers"` · `"tensorflow"`

Cache root: `~/cache/kappa-framework/datasets/{name}_{version}/`

---

## Benchmarks

| Method | HTTP |
|---|---|
| `load_benchmark(benchmark_id)` | Returns `Benchmarks` handle (loads details via v2 benchmark API) |
| `filter_benchmarks(...)` / `list_benchmarks()` | `GET /model-micro-services/v2/benchmarks` |
| `get_benchmark(id)` / `create_benchmark` / `update_benchmark` / `delete_benchmark` | `…/benchmarks[/{id}]` |
| `complete_benchmark_inference(id, model_version_id)` | `POST …/benchmarks/inferences/{id}/{version_id}` — `version_id` must be an existing model version, else `404 MODEL_VERSION_NOT_FOUND` |
| `download_benchmark_dataset_package(id, dataset_id?, version_no?, dataset_path?)` | dataset package → benchmark proxy → legacy zip |
| `get_benchmark_dataset_package_manifest(id)` | `GET …/benchmarks/datasets/{id}/package` |
| `list_benchmark_remarks` / `add_benchmark_remark` | `…/benchmarks/{id}/remarks` |
| `get_benchmark_review` / `save_benchmark_review` / `finalize_benchmark_review` | `…/benchmarks/review/{expert_id}/{id}` |
| `get_benchmark_report` / `download_benchmark_report` / `regenerate_benchmark_report` | `…/benchmarks/report*` |
| `respond_to_benchmark_expert_request` / `assign_benchmark_expert` | `…/benchmarks/management*` |
| `propose_benchmark_dataset` / `confirm_benchmark_dataset` / `reject_benchmark_dataset` | `…/benchmarks/management/expert/{id}/dataset*` |
| `get_benchmark_dataset_attachments` / `get_benchmark_flow_schema` | `…/benchmarks/datasets/attachments/…`, `…/benchmarks/flow-schema` |

See [Benchmarks.md](Benchmarks.md).

---

## Model artifacts

| Method | HTTP |
|---|---|
| `write_model_inference(model_id, predictions?, metrics?, inference_result?, artifacts?, …)` | schema → validate → `POST …/models/inferences/{model}` → artifact uploads |
| `upload_model_artifacts(model_id, inference_id, paths, …)` | one upload per file, transport chosen per size |
| `upload_model_inference_file(model_id, inference_id, file_path, file_category?, replace?, use_session?)` | `POST`/`PATCH …/models/inferences/files/{model}/{inference}` — switches to an upload session for large files |
| `upload_model_artifact_session(..., on_progress?, checksum?, resume?, max_retries?, wait_for_slot?)` | `…/upload-session` → `…/part-urls` → presigned `PUT` → `…/complete` |
| `get_model_artifact_upload_session` / `abort_model_artifact_upload_session` | `…/upload-session/{upload_id}` |
| `get_model_inference_artifacts_package` / `get_model_version_artifacts_package` | `GET …/package` / `…/artifacts/package` |
| `download_model_inference_artifact_file(..., redirect?)` | `GET …/inferences/files/{model}/{inference}/{file_id}` |
| `download_model_inference_artifacts_package(..., dest_dir)` | manifest + per-file download |
| `download_model_inference_artifacts` / `download_model_version_artifacts` | `GET …/zip` — small packages only |

`file_category`: `1` Training · `2` Inference · `3` Model · `4` Data · `5` Other. The plain upload defaults to `2`; upload sessions and the two helpers above default to `3`.

Oversized plain uploads are rejected with `413 USE_KAPPA_APK`, and packages past the zip ceiling refuse the zip route with `409 PACKAGE_TOO_LARGE_FOR_ZIP`. The SDK handles both: it moves large uploads onto a session automatically, and `download_model_inference_artifacts_package` fetches each file separately. Sessions need object storage enabled server-side (`503` otherwise) and allow only a couple of concurrent uploads per model (`429`), which `wait_for_slot` waits out; abort abandoned sessions to free a slot sooner.

### Writing an inference with its artifacts

`write_model_inference` is the one-call path: it reads the model's effective inference schema, shapes the document, validates it, creates the inference and uploads the artifacts.

```python
written = client.write_model_inference(
    model_id,
    predictions=[
        {"entityId": item.entity_id, "original": item.annotations,
         "predicted": {"class_name": "pizza", "confidence": 0.98}}
    ],
    metrics={"accuracy": 0.93},
    artifacts=["./checkpoints", "./config.json"],   # files and/or directories
    on_progress=lambda name, sent, total, pct: print(f"{name} {pct}%"),
)
print(written["inferenceId"], written["schema"]["kind"])
```

Prediction dicts accept `entity_id` or `entityId`, and a bare label string for `predicted` is wrapped into the key the schema requires (`class_name`, `text`, `answer`, …). Pass `inference_result=…` to send a document you built yourself. A payload the schema rejects raises `ValueError` listing the failing JSON paths before anything is written.

### Large weights (KappaApk-only path)

The GUI caps artifact uploads; multi-GB model weights can only be uploaded through this SDK. Both helpers handle them:

```python
# Whole shard directory into an existing inference; re-run to fill any gaps.
client.upload_model_artifacts(
    model_id, inference_id, "./llm-70b",     # model-00001-of-00042.safetensors, …
    on_progress=lambda name, sent, total, pct: print(f"{name} {pct}%"),
)

client.upload_model_inference_file(
    model_id, inference_id, "weights.safetensors", file_category=3,
)   # 8 GB file → upload session, transparently

paths = client.download_model_inference_artifacts_package(
    model_id, inference_id, "./artifacts",
)
```

Sessions retry each part against a freshly presigned URL and checkpoint their ETags under the user cache dir, so an interrupted 100 GB shard resumes where it stopped instead of starting over (`resume=False` to disable). `upload_model_artifacts` uploads one file at a time to stay inside the server's admission limit and, with `skip_existing` (default), skips artifacts already attached with the same name and size.

---

## Example

```python
with KappaApkClient(gateway, user, pwd) as client:
    page = client.list_datasets(size=50)
    ds = client.get_dataset_details(dataset_name="MNIST")
    loader = client.get_dataset_loader(dataset_name="MNIST", version_no="1.0.0", batch_size=64)
    for batch in loader:
        process(batch)
```

[← Index](README.md) · [Data models](DataModels.md)
