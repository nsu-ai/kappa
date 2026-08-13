# Datasets

**Русская версия:** [ru/Datasets.md](ru/Datasets.md)

Guide to dataset operations with `kappa_apk`. For HTTP path details see [KappaApkClient.md](KappaApkClient.md).

---

## List and search

```python
# Paginated JSON
page = client.list_datasets(page=1, size=50, order_by="modifiedOn", order_keyword="DESC")
items = page["items"]

# Typed
datasets = client.list_datasets_typed()

# Rich filter
results = client.filter_datasets(
    dataset_tags="Image Classification",
    dataset_type=1,
    dataset_status=1,
    publish_type=1,
    search="pizza",
    page=1,
    size=20,
)

# Only what you own / are assigned / were shared, with per-item build status
mine = client.filter_datasets(query_all=False, selected_version_no="1.0.0")
for item in mine["items"]:
    print(item["datasetName"], item.get("selectedVersionBuildStatus"))
```

Pages are 1-based and the envelope is `{items, total, page, size, pages}`. `query_all` defaults to `True`, which also lists the public catalogue. Passing `selected_version_id` or `selected_version_no` adds `selectedVersionNo` and `selectedVersionBuildStatus` to each item — cheaper than a version call per dataset when you only need to know whether a version is `ready`.

---

## Create, update, delete

```python
from kappa_apk import NewDataset, UpdateDatasetRequest, join_ml_tags

# Tags: first must be a predefined ML tag for dataset_type (catalog:
# GET …/system/config/dataset_tags_{type}). Extra custom tags are allowed after it.
result = client.add_dataset(NewDataset(
    dataset_name="MyDataset",
    dataset_type=1,  # Computer Vision → dataset_tags_1
    dataset_short_info="Image classification",
    dataset_tags=join_ml_tags("Image Classification", "my-project", "batch-a"),
    dataset_verification_type=1,
))

client.update_dataset(42, UpdateDatasetRequest(dataset_name="MyDataset-v2", remark="Renamed"))
client.delete_dataset(42, remark="Obsolete")   # soft-delete
```

**Tag rules (backend + FE):** catalog is `GET …/system/config/dataset_tags_{dataset_type}` (same list for model create). Backend requires ≥1 predefined ML tag (`ml_tags_config_fields_info` after normalizing display labels). Put that predefined tag **first** — Label Studio / CVAT use the first predefined tag in order. Custom tags (`torchvision`, project names, …) may follow.

```python
from kappa_apk import join_ml_tags

# Or: client.list_predefined_ml_tags(1)  → ["Image Classification", …]
tags = join_ml_tags("Image Classification", "torchvision", "my-batch")
client.add_dataset(NewDataset(..., dataset_type=1, dataset_tags=tags))
# Same for models: create_model({..., "mlModelType": 1, "mlModelTags": tags})
```

---

## Labels

```python
client.add_dataset_labels(42, ["cat", "dog", "bird"])
labels = client.get_dataset_labels(42)
names = client.get_dataset_label_names(42)
client.update_dataset_label(42, label_id=7, label="feline")
```

---

## Entities (upload samples)

```python
from kappa_apk import NewDatasetEntity, UpdateDatasetEntity, DeleteDatasetEntities

client.add_dataset_entity(
    dataset_id=42,
    entity=NewDatasetEntity(
        ds_entity_name="sample_001",
        collected_on="2026-01-15T10:00:00",
        labeling_algo="manual",
        ds_entity_info={"label": "cat"},
    ),
    file_paths=["/data/img001.jpg"],   # files, directory, or URLs
)

client.update_dataset_entity(42, "entity-uuid", UpdateDatasetEntity(remark="Fixed label"))
entities = client.list_dataset_entities(42, version_id=7)
page = client.filter_dataset_entities(42, entity_name="sample", page=1, size=20)

# Only entities assigned to you, newest first
mine = client.filter_dataset_entities(
    42, assignment_filter="assigned", order_by="modifiedOn", order="DESC",
)

client.delete_dataset_entities(["uuid-1", "uuid-2"], remark="Duplicates removed")
```

Entity pages are **1-based** like dataset pages, and sort direction uses `order` (not `order_keyword`). You can also pass `entity_id`, `location_id`, `start_date` and `end_date`. There is no server-side split filter — read `dsEntityInfo.split` from each item.

Pass `file_category="input"|"output"` and optional `split="train"|"validation"|"test"` on create/update. Single entity files are capped at **2 GB** each.

---

## Bulk upload (requires Kappa ≥ 2.11.0)

Async job API — same flow as the React bulk dialog. Full script: [`code_examples/bulk_upload.py`](../code_examples/bulk_upload.py).

| Upload | Max size | Notes |
|---|---|---|
| CSV | **2 GB** client | Backend default is **50 MB** (`BULK_UPLOAD_MAX_CSV_BYTES`); raise that env for larger CSVs |
| Archive `.zip` | **50 GB** | `upload_type="archive"` + `archive_layout` + `dataset_schema` |

`archive_layout` must be `input_output` or `classes`. For `input_output`, pass `dataset_schema={"inputDataPath": "input", ...}` where paths are **relative to the zip root** (include a parent folder if the zip wraps one, e.g. `"abc/input"`). For `classes`, pass a non-empty `classes` list. Do **not** send an empty `{}` schema — the server treats that as missing and returns 422. The file is **streamed** from disk (not loaded fully into RAM).

If folder paths are wrong, the backend may respond with HTTP 400 and `status=needs_correction` plus `jobId` / `availableArchiveDirectories`. The SDK returns that JSON (does not raise) so you can call `retry_bulk_upload_job` with corrected `datasetSchema` without re-uploading the zip. `wait_for_bulk_upload_job` also stops on `needs_correction`.

Also see [`code_examples/dataset_operations_example.py`](../code_examples/dataset_operations_example.py), [`code_examples/dataset_lifecycle_example.py`](../code_examples/dataset_lifecycle_example.py), and [`code_examples/bulk_mutation_and_version_build.py`](../code_examples/bulk_mutation_and_version_build.py) (mutations + version builds).

```python
from kappa_apk import compatibility_info, min_backend_version

print(min_backend_version())   # "2.11.0"
print(compatibility_info())

# Optional RBAC check (also available via check_permission=True on upload)
if not client.has_permission("dataset.write", dataset_id=42):
    print(client.get_my_permissions(dataset_id=42))
    raise SystemExit("no write access")

def on_upload(sent, total, percent):
    print(f"transfer {percent}%")

start = client.bulk_upload_dataset_entities(
    dataset_id=42,
    file_path="/data/entities.zip",
    upload_type="archive",
    labeling_algo="default",
    source="lab-batch-1",
    archive_layout="input_output",  # or "classes"
    dataset_schema={"inputDataPath": "input"},  # required for input_output
    strict=True,
    on_upload_progress=on_upload,
)
job_id = start["jobId"]

def on_job(job):
    # BulkUploadJob: status, phase, processed_rows, total_rows, percent, can_retry, …
    print(job.status, job.phase, job.percent, f"{job.processed_rows}/{job.total_rows}")

final = client.wait_for_bulk_upload_job(
    42, job_id, poll_interval_secs=2, timeout_secs=3600, on_progress=on_job
)
print(final.status, final.can_retry, final.as_dict())

# Or poll manually:
job = client.get_bulk_upload_job(42, job_id)
if job.is_terminal():
    print(job.status)
```

Helpers: `list_bulk_upload_jobs`, `cancel_bulk_upload_job`, `retry_bulk_upload_job`, `cancel_stale_bulk_upload_jobs`.

HTTP **403** raises `PermissionError` with a hint to inspect permissions. **429** means admission control — retry later.

---

## Bulk mutations (requires Kappa ≥ 2.11.0)

Self-verify, auto-verify, mark-labeled, delete/recover/delete-files enqueue an async **bulk-mutation** job (`202` + `jobId`). Poll with `BulkMutationJob` helpers. Only one active mutation per dataset (HTTP **409** if busy).

```python
stats = client.get_mark_labeled_stats(42)
start = client.mark_dataset_entities_labeled(42, all_eligible=True, remark="batch")
# or: client.mark_dataset_entities_labeled(42, dataset_entity_ids=["…"])
# Inline ID lists are capped at BULK_MUTATION_INLINE_ID_LIMIT (default 5000) → HTTP 413.
# For whole-dataset runs use all_eligible=True.

def on_mut(job):
    print(job.job_type, job.status, job.percent, job.processed_count, job.total_count)

final = client.wait_for_bulk_mutation_job(42, start["jobId"], on_progress=on_mut)
print(final.status, final.succeeded_count, final.error_detail)
# Default timeout is 1 hour; raise timeout_secs for very large datasets.

# Self-verify (status 1=Pass, 3=Needs Modification + comment) / auto-verify
client.bulk_self_verify_dataset_entities(42, status=1)
client.auto_verify_dataset_entities(42)

# Delete / recover / delete-files also return jobId — wait the same way
```

Helpers: `get_bulk_mutation_job`, `list_bulk_mutation_jobs`, `cancel_bulk_mutation_job`, `get_self_verify_stats`.

Full script: [`code_examples/bulk_mutation_and_version_build.py`](../code_examples/bulk_mutation_and_version_build.py).

---

## Versions

On Kappa ≥ 2.11, create/refresh **enqueue an archive build**. Wait until the build completes before publish or download.

```python
from kappa_apk import NewDatasetVersion

created = client.create_dataset_version(
    42, NewDatasetVersion(version_availability=1, version_remark="Initial")
)
# created: versionNo, versionId, jobId, buildStatus
build = client.wait_for_version_build_job(created["jobId"], timeout_secs=3600)
assert build.is_ready()

client.publish_dataset_version(42, created["versionNo"], publish_type=2)
versions = client.list_dataset_versions(42)
client.delete_dataset_version(42, "0.9.0")

details = client.get_dataset_version_details(dataset_name="MyDS", version_no="1.0.0")
```

Helpers: `get_version_build_job`, `retry_version_build_job`, `refresh_dataset_version` (also returns `jobId`).

---

## Download & cache

Use `download_dataset_version_package` — it reads the package manifest and pulls shards, falling back to the legacy single zip when the backend has no package (pre-2.11 or single-shard builds).

Archives download to `~/cache/kappa-framework/datasets/{dataset_name}_{version_no}/`. Repeat calls skip re-download if the cache directory exists.

```python
# Manifest + shards, with legacy fallback
info = client.download_dataset_version_package(dataset_name="MyDS", version_no="1.0.0")
print(info.data_path)

# Legacy single zip only (fails with 404 for multi-shard versions)
info = client.download_dataset_version_archive(dataset_name="MyDS", version_no="1.0.0")
```

`load_kappa_dataset` and `get_dataset_loader` use the package path automatically, so training flows work for sharded versions without changes.

On Kappa ≥ 2.11 the backend only keeps the single zip when a version fits in one shard (default 2 GB / 50 000 files), so large versions **must** use the package path.

Publish/download before `buildStatus=ready` → HTTP **409** `VERSION_ARCHIVE_NOT_READY`.

Zip extraction validates paths (zip-slip protection via prefix check on extract paths).

---

## Load for training

```python
dataset = client.load_kappa_dataset(
    dataset_name="MyDS",
    version_no="1.0.0",
    transform=my_transform,
    transform_input_mode="content",   # or "path"
)

loader = client.get_dataset_loader(
    dataset_name="MyDS",
    version_no="1.0.0",
    loader_type="kappa",
    batch_size=32,
    shuffle=True,
)

for epoch in range(10):
    for batch in loader:
        x = batch[0]   # dict with entity_id + transformed fields
```

See [LoadersAndTransforms.md](LoadersAndTransforms.md) for loader types, dropout, and transforms.

[← Index](README.md)
