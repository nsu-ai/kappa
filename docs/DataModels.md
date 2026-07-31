# Data models

**Русская версия:** [ru/DataModels.md](ru/DataModels.md)

Typed classes exported from `kappa_apk`. Request models implement `to_api_json()` for camelCase API bodies.

---

## Read-only (API responses)

### `Dataset`

Dataset metadata from listing / filter / lookup.

| Property | Type | Notes |
|---|---|---|
| `dataset_id` | `int` | |
| `dataset_name` | `str` | |
| `dataset_type` | `int` | 1 Vision · 2 Text · 3 Audio |
| `dataset_type_interp` | `str` | Human-readable type |
| `dataset_short_info` | `str` | |
| `dataset_status` | `int` | |
| `dataset_status_interp` | `str` | |
| `dataset_tags` | `str` | Comma-separated. **First** tag must be a predefined ML tag for `dataset_type` (from `dataset_tags_{type}`); custom tags may follow |
| `publish_type` | `int` | 0 Not Published · 1 Private · 2 Open Source · 3 Public on Demand · 4 Purchase |
| `created_on` / `modified_on` | `str` | |
| `version_no` | `str \| None` | Latest version string |

### `DatasetVersionDetails`

| Property | Type |
|---|---|
| `id`, `user_id`, `dataset_id` | `int` |
| `version_availability` | `int` (1 = available) |
| `version_no`, `version_remark` | `str` |
| `publish_type` | `int` |
| `created_on`, `modified_on` | `str` |

### `DatasetDownloadDetails`

Result after archive download + extract.

| Property | Type |
|---|---|
| `dataset_id` | `int` |
| `version_no` | `str` |
| `data_path` | `str` | Local cache directory |
| `download_status` | `bool` | |

### `DatasetItem` / `ItemFile`

One sample in a dataset version.

```python
item.entity_id          # str
item.files              # list[ItemFile] | None
item.annotations        # list[dict] | None

file.file_id            # str
file.file_name          # str
file.file               # str — absolute path on disk
```

### `DatasetLabel`

| Property | Type |
|---|---|
| `label_id`, `dataset_id` | `int` |
| `label` | `str` |
| `created_on`, `modified_on` | `str \| None` |

### `FileInformation` / `BenchmarkResult`

Used in benchmark submissions. See [Benchmarks.md](Benchmarks.md).

---

## Request bodies (mutable)

| Class | Use |
|---|---|
| `NewDataset` | `add_dataset` |
| `UpdateDatasetRequest` | `update_dataset` |
| `NewDatasetEntity` | `add_dataset_entity` |
| `UpdateDatasetEntity` | `update_dataset_entity` |
| `UpdateDatasetLabel` | `update_dataset_label` |
| `DeleteDatasetEntities` | `delete_dataset_entities` |
| `NewDatasetVersion` | `create_dataset_version` |

All accept equivalent plain `dict` with camelCase keys at the client boundary.

### Example: `NewDataset`

```python
from kappa_apk import NewDataset, join_ml_tags

ds = NewDataset(
    dataset_name="MyDataset",
    dataset_type=1,
    dataset_short_info="Vision classification",
    dataset_tags=join_ml_tags("Image Classification", "cls"),
    dataset_verification_type=1,
)
client.add_dataset(ds)  # check_tags=True by default
```

### Example: `NewDatasetEntity`

```python
from kappa_apk import NewDatasetEntity

entity = NewDatasetEntity(
    ds_entity_name="img_001",
    collected_on="2026-01-15T10:00:00",
    labeling_algo="manual",
    ds_entity_info={"label": "cat"},
)
client.add_dataset_entity(42, entity, file_paths=["/data/img.jpg"])
```

---

## Runtime objects

| Class | Role |
|---|---|
| `KappaDataset` | In-memory dataset (`__len__`, `__getitem__`) |
| `KappaDataLoader` | Batch iterator with epoch reshuffle |
| `Benchmarks` | Returned by `load_benchmark()` — not a top-level import |
| `BenchmarkVerification` | Standalone verification helper |
| `DataLoaderHelper` | Static utilities (`peek_batch`, TF signature inference) |

[← Index](README.md) · [Datasets guide](Datasets.md)
