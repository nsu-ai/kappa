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
| `filter_datasets(search?, dataset_tags?, dataset_type?, dataset_status?, publish_type?, page?, size?, order_by?, order?)` | `GET …/datasets/filter` | Rich filters (defaults: page=1, size=20) |
| `get_dataset_details(dataset_id?, dataset_name?)` | `GET …/datasets/filter` | Single `Dataset` |
| `get_dataset_fields(dataset_id)` | `GET …/datasets/fields/{id}` | Input field schema |

`list_datasets` and `filter_datasets` hit the same filter endpoint with **different default pagination**.

---

## Dataset CRUD

| Method | HTTP |
|---|---|
| `add_dataset(dataset)` | `POST …/datasets/new` |
| `update_dataset(dataset_id, update)` | `PUT …/datasets/{id}` |
| `delete_dataset(dataset_id, remark)` | `DELETE …/datasets/{id}` (soft-delete; recover via server `/datasets/recover`) |

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
| `filter_dataset_entities(dataset_id, entity_name?, entity_status?, version_id?, page?, size?, order_by?, order?)` | `GET …/datasetEntities/filter/{id}` |
| `delete_dataset_entities(dataset_entity_ids, remark, version_id?)` | `DELETE …/datasetEntities` |

`file_paths` accepts file paths, a directory (immediate children), or `http(s)://` URLs.

---

## Versions & archives

| Method | HTTP |
|---|---|
| `get_dataset_version_details(dataset_id?, dataset_name?, version_id?, version_no?)` | `POST …/versions/details/list` or `GET …/versions/{id}/{ver}` |
| `download_dataset_version_archive(..., dataset_path?)` | `GET …/versions/archive/{id}/{ver}` |
| `create_dataset_version(dataset_id, version)` | `POST …/versions/new/{id}` |
| `list_dataset_versions(dataset_id, version_availability?)` | `GET …/versions/{id}` |
| `delete_dataset_version(dataset_id, version_no)` | `DELETE …/versions/{id}/{ver}` |
| `publish_dataset_version(dataset_id, version_no, publish_type)` | `POST …/versions/publish/{id}/{ver}?publish_type=N` |

`publish_type`: `0` Private · `1` Internal · `2` Public

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

See [Benchmarks.md](Benchmarks.md).

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
