# KappaApk — Internal Developer Reference

> Maintainer reference — align with `src/` before releases. Use as context instead of re-reading code.
> Last updated: 2026-07-02 | Synced tag: `kappa-apk-3.0.0-beta` | Branch: `dev`
>
> **Related docs:**
> - [SYNC_TRACKING.md](SYNC_TRACKING.md) — upstream tag sync history and procedure
> - [FEATURE_CODE_MAPPING.md](FEATURE_CODE_MAPPING.md) — legacy `kf_sdk` → `kappa_apk` API mapping
>
> **Feedbacks source:** full `Feedbacks` implementation lives on branch **`dataset-feedback-code-backup`** (`feedbacks.rs`, `feedbacks_threshold.rs`, `feedbacks_attribution.rs`). Tag `kappa-apk-3.0.0-beta` ships `KappaDataLoader` dropout hooks only; merge from that branch when integrating Feedbacks.

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Repository Structure](#2-repository-structure)
3. [Rust Library — File-by-File Reference](#3-rust-library--file-by-file-reference)
   - [src/lib.rs](#srclibers)
   - [src/traits.rs](#srctraitrs)
   - [src/client.rs](#srcclientrs)
   - [src/models/users_model.rs](#srcmodelsusersmoderl)
   - [src/models/login_models.rs](#srcmodelsloginmodelsrs)
   - [src/models/datasets_model.rs](#srcmodelsdatasetsmoderl)
   - [src/models/benchmarks_model.rs](#srcmodelsbenchmarksmoderl)
   - [src/datasets/datasets.rs](#srcdatasetsdatasetsrs)
   - [src/datasets/kappa_dataloader.rs](#srcdatasetskappa_dataloaderrs)
   - [src/datasets/feedbacks.rs](#srcdatasetsfeedbacksrs)
   - [src/datasets/feedbacks_threshold.rs](#srcdatasetsfeedbacks_thresholdrs)
   - [src/datasets/feedbacks_attribution.rs](#srcdatasetsfeedbacks_attributionrs)
   - [src/transforms/vision.rs](#srctransformsvisionrs)
4. [System Architecture & Design Patterns](#4-system-architecture--design-patterns)
5. [Key Data Flows](#5-key-data-flows)
6. [Public API Surface](#6-public-api-surface)
7. [Configuration Reference](#7-configuration-reference)
8. [Dependencies](#8-dependencies)
9. [Notable Implementation Details](#9-notable-implementation-details)
10. [Feedbacks — Algorithm & Literature Reference](#10-feedbacks--algorithm--literature-reference)
11. [Code Review Backlog](#11-code-review-backlog) → [CODE_REVIEW_BACKLOG.md](CODE_REVIEW_BACKLOG.md)
12. [Sync & Mapping Docs](#12-sync--mapping-docs)

---

## 1. Project Overview

**KappaApk** is a Rust+PyO3 client SDK for interacting with **Kappa-framework** — a self-hosted microservices platform for ML/AI research workflows. Distributed as the `kappa_apk` Python package (`pip install kappa-apk`).

**This repo** (`github.com/nsu-ai/kappa`) is the public distribution of the SDK, synced from the private KappaApk source. See [SYNC_TRACKING.md](SYNC_TRACKING.md).

| Layer | Technology | Role |
|---|---|---|
| Rust library | PyO3 + Tokio + reqwest | High-performance HTTP client, dataset I/O, data loaders |

**Kappa-framework** (located at `/media/ganga/NSU/Projects/Kappa/kappa-framework`) is the backend platform this SDK targets. It is composed of:

| Service | Path in framework | Role |
|---|---|---|
| Traefik Gateway | `dev/traefik` | API gateway — single entry point for all services |
| User service | `dev/xframework-user-services` | Auth, sessions, orgs (`/user-micro-services/v2`) |
| Dataset service | `dev/xframework-dataset-services` | Dataset CRUD, versioning, entity storage (`/data-micro-services/v2`) |
| Model service | `dev/xframework-model-services` | Model registry and benchmarks |
| CVAT integration | `dev/xframework-cvat-services` | Annotation via CVAT |
| Label Studio integration | `dev/xframework-label-studio-services` | Annotation via Label Studio |
| Storage | PostgreSQL + MongoDB + KeyDB + MinIO | Persistence layer |

**KappaApk** communicates exclusively through the Traefik gateway, so `base_url` should point to the Traefik gateway host.

---

## 2. Repository Structure

```
KappaApk/
├── src/                          # Rust library source
│   ├── lib.rs                    # Module registration, Python exports
│   ├── traits.rs                 # ApiClient trait (core abstraction)
│   ├── client.rs                 # KappaApkClient implementation
│   ├── models/
│   │   ├── users_model.rs        # User, UserTypeDetails, OrgDetails
│   │   ├── login_models.rs       # LoginRequest
│   │   ├── datasets_model.rs     # Dataset, DatasetItem, NewDataset, UpdateDataset*, etc.
│   │   └── benchmarks_model.rs   # Benchmark, Results, Predictions, FileInformation
│   ├── datasets/
│   │   ├── mod.rs                # Module exports
│   │   ├── datasets.rs           # Datasets struct — all dataset API calls
│   │   ├── kappa_dataloader.rs   # KappaDataLoader (epoch-aware; entity dropout hooks)
│   │   ├── dataloader_helper.rs  # DataLoaderHelper (peek_batch, TF signature inference)
│   │   ├── feedbacks.rs          # Feedbacks — on branch `dataset-feedback-code-backup`
│   │   ├── feedbacks_threshold.rs # Threshold / EMA / GMM — same branch
│   │   ├── feedbacks_attribution.rs # BLESS entity_scores — same branch
│   │   └── dataset_entities.rs   # **Empty stub** on mainline — no implementation yet
│   ├── benchmarks/
│   │   ├── mod.rs
│   │   ├── benchmarks.rs         # Benchmarks workflow (load, dataset, save, submit)
│   │   └── verifications.rs      # BenchmarkVerification (v2 verification endpoints)
│   ├── users/
│   │   └── users.rs              # get_user_profile — dead code; v2 `GET /users/me`
│   ├── transforms/
│   │   ├── vision.rs             # Image transforms (Compose, resize, crop, normalize)
│   │   ├── text.rs               # Text transforms
│   │   └── audio.rs              # Audio transforms
│   └── utils/
│       ├── file_utils.rs         # Path, hashing, directory helpers
│       ├── git_utils.rs          # Git blob hashing for file fingerprints
│       ├── python_json.rs        # PyObject ↔ serde_json::Value bridge
│       ├── zip_utils.rs          # Zip helpers — **currently unused**
│       └── metric_utils.rs       # Metric helpers
├── code_examples/                # Usage examples
├── Cargo.toml                    # Rust dependencies
├── pyproject.toml                # Maturin build config
└── build_wheel.sh                # Build script
```

---

## 3. Rust Library — File-by-File Reference

### src/lib.rs

**Role**: Module entry point and Python module registration.

**Exports to Python (`kappa_apk` package)**:
```
version() → reads CARGO_PKG_VERSION at compile time (currently "2.0.0-beta"; release tag: kappa-apk-3.0.0-beta)

# Core client
KappaApkClient         — main HTTP client

# Dataset containers and loaders
KappaDataset           — in-memory dataset (PyTorch-style __len__/__getitem__)
KappaDataLoader        — epoch-aware Rust data loader (dropout hooks on mainline)
DataLoaderHelper       — static loader utilities (peek_batch, infer_tf_output_signature)
Feedbacks              — training loss feedback tracker (**branch `dataset-feedback-code-backup`**)
BenchmarkVerification  — standalone benchmark file verification (v2 API, public — no JWT)

# Dataset response models (immutable)
Dataset                — dataset metadata record
DatasetVersionDetails  — version metadata
DatasetDownloadDetails — local cache result after archive download
DatasetItem            — single labelled sample
ItemFile               — file within a sample
DatasetLabel           — single label record (from labels endpoint)

# Dataset request/mutation models (mutable)
NewDataset             — create dataset request
UpdateDatasetRequest   — update dataset metadata request
NewDatasetEntity       — create entity request
UpdateDatasetEntity    — update entity request
UpdateDatasetLabel     — rename label request
DeleteDatasetEntities  — bulk entity soft-delete request
NewDatasetVersion      — create version request

# Transform submodules
vision                 — image transform submodule
text                   — text transform submodule
audio                  — audio transform submodule
```

**Internal modules declared**: `client`, `traits`, `models`, `users`, `benchmarks`, `utils`, `datasets`, `transforms`

**Branch split — Feedbacks:**
On **`v2.0.0`** (mainline), `Feedbacks` is not registered in `lib.rs`. On **`dataset-feedback-code-backup`**, `lib.rs` adds `mod feedbacks` and exports `Feedbacks` to Python. To work on or build Feedbacks:

```bash
git fetch origin dataset-feedback-code-backup
git checkout dataset-feedback-code-backup   # or merge/cherry-pick into your branch
```

See §3 (`feedbacks*.rs`) and §10 for full API / algorithm reference.

---

### src/traits.rs

**Role**: Core `ApiClient` trait that all HTTP clients must implement. Enables pluggable implementations (real client, mock client).

**Trait: `ApiClient`**

| Method | Signature | Description |
|---|---|---|
| `get_base_url` | `() -> String` | API root URL |
| `get_user_id` | `() -> Option<i32>` | Authenticated user ID |
| `get_user_type_id` | `() -> Option<i32>` | User classification ID |
| `get_token` | `() -> Option<String>` | JWT bearer token |
| `get_http_client` | `() -> reqwest::Client` | HTTP client instance |
| `get_runtime` | `() -> Arc<tokio::runtime::Runtime>` | Shared Tokio runtime |
| `make_request` | `(method, endpoint, data, token) -> PyResult<PyObject>` | Generic HTTP request → Python dict |
| `fetch_url_bytes` | `(url) -> PyResult<Vec<u8>>` | Binary file download |
| `submit_dataset_entity_request` | `(method, endpoint, json_field_name, json_value, file_parts, token) -> PyResult<PyObject>` | Multipart form+file upload |

**Default implementations (no override needed)**:

| Method | Description |
|---|---|
| `is_authenticated()` | Returns `get_token().is_some()` |
| `user_id()` | Returns `get_user_id().unwrap_or(0)` |
| `user_type_id()` | Returns `get_user_type_id().unwrap_or(0)` |
| `require_token()` | Returns token or raises `PyValueError("Not authenticated")` |

---

### src/client.rs

**Role**: Primary HTTP client. Implements `ApiClient`. Exposed to Python as `KappaApkClient`.

**Struct Fields**:

| Field | Type | Description |
|---|---|---|
| `client` | `reqwest::Client` | Reusable HTTP client |
| `runtime` | `Arc<tokio::runtime::Runtime>` | Tokio async runtime (1 per client instance) |
| `base_url` | `String` | API root URL |
| `login_id` | `String` | Email or username |
| `passwd` | `String` | Password (not cleared after login) |
| `login_response` | `Option<User>` | Cached login state (contains token) |

**Methods**:

| Method | Endpoint / Behavior | Returns |
|---|---|---|
| `new(base_url, login_id, passwd)` | Creates client, does NOT connect | `PyResult<Self>` |
| `connect()` | POST `/session/new` | Python dict (snake_case user+token) |
| `close()` | DELETE `/session/` | `()` |
| `make_request(method, endpoint, data, token)` | GET/POST/PUT/DELETE with JSON | `PyObject` |
| `submit_dataset_entity_request(...)` | Multipart form upload | `PyObject` |
| `list_datasets(page, size, order_by, order_keyword)` | Calls `Datasets::list_datasets_json` | `PyObject` |
| `list_datasets_typed(...)` | Same, returns `List[Dataset]` pyclass objects | `PyObject` |
| `get_dataset_version_details(dataset_id?, dataset_name?, version_id?, version_no?)` | Calls `Datasets::get_dataset_version_details` | `DatasetVersionDetails` |
| `download_dataset_version_archive(dataset_id?, dataset_name?, version_id?, version_no?, dataset_path?)` | Downloads + extracts ZIP to cache | `DatasetDownloadDetails` |
| `load_kappa_dataset(...)` | Download + parse → KappaDataset | `Py<KappaDataset>` |
| `get_dataset_loader(..., loader_type?)` | Wraps KappaDataset in loader | `PyObject` |
| `add_dataset(dataset)` | POST `/datasets/new` | `PyObject` |
| `update_dataset(dataset_id, update)` | PUT `/datasets/{dataset_id}` | `PyObject` |
| `add_dataset_entity(dataset_id, entity, file_paths?)` | Multipart POST entity+files | `PyObject` |
| `update_dataset_entity(dataset_id, entity_id, update, file_paths?)` | Multipart PUT entity+files | `PyObject` |
| `get_dataset_details(dataset_id?, dataset_name?)` | Calls `Datasets::get_dataset_details` | `Dataset` |
| `filter_datasets(search?, dataset_id?, dataset_name?, dataset_type?, dataset_tags?, dataset_status?, publish_type?, page?, size?, order_by?, order_keyword?)` | GET `/datasets/filter` with all params | `PyObject` |
| `get_dataset_fields(dataset_id)` | GET `/datasets/fields/{id}` | `PyObject` |
| `delete_dataset(dataset_id, remark?)` | Soft-delete (PUT with `datasetStatus=0`) | `PyObject` |
| `add_dataset_labels(dataset_id, labels)` | POST `/datasets/labels/{id}` | `PyObject` |
| `get_dataset_labels(dataset_id)` | GET `/datasets/labels/{id}` | `Vec<DatasetLabel>` |
| `update_dataset_label(dataset_id, label_id, label)` | PUT `/datasets/labels/{id}` | `PyObject` |
| `list_dataset_entities(dataset_id, version_id?)` | GET `/datasets/datasetEntities/{id}` | `PyObject` |
| `get_dataset_entity(dataset_id, entity_id, version_id?)` | GET `/datasets/datasetEntities/{id}/{eid}` | `PyObject` |
| `filter_dataset_entities(dataset_id, entity_name?, entity_status?, version_id?, page?, size?, order_by?, order?)` | GET `/datasets/datasetEntities/filter/{id}` | `PyObject` |
| `delete_dataset_entities(dataset_entity_ids, remark, version_id?)` | DELETE `/datasets/datasetEntities` (bulk) | `PyObject` |
| `create_dataset_version(dataset_id, version)` | POST `/datasets/versions/new/{id}` | `PyObject` |
| `list_dataset_versions(dataset_id, version_availability?)` | GET `/datasets/versions/{id}` | `PyObject` |
| `delete_dataset_version(dataset_id, version_no)` | DELETE `/datasets/versions/{id}/{ver}` | `PyObject` |
| `publish_dataset_version(dataset_id, version_no, publish_type)` | POST `/datasets/versions/publish/{id}/{ver}` | `PyObject` |
| `load_benchmark(benchmark_id)` | Loads benchmark result | `Py<Benchmarks>` |
| `__enter__()` | Calls `connect()` | `Self` |
| `__exit__(exc_type, exc_val, exc_tb)` | Calls `close()`, ignores errors | `()` |

**Properties**: `get_base_url`, `set_base_url`, `get_token`, `is_authenticated`

**Helper functions (module-level, not public)**:

| Function | Description |
|---|---|
| `json_value_to_pyobject(py, value)` | Converts `serde_json::Value` → Python dict/list/primitive |
| `py_json_dumps(v)` | Converts Python object → JSON string (calls `json.dumps`) |
| `encode_new_dataset_payload(v)` | Serializes `NewDataset` or generic dict to JSON |
| `encode_update_dataset_payload(v)` | Serializes `UpdateDatasetRequest` or generic dict |
| `encode_new_entity_payload(v)` | Serializes `NewDatasetEntity` or generic dict |
| `encode_update_entity_payload(v)` | Serializes `UpdateDatasetEntity` or generic dict |
| `encode_new_version_payload(v)` | Serializes `NewDatasetVersion` or generic dict |

**`make_request` behavior**:
- Adds headers: `Accept: application/json`, `Content-Type: application/json`, `Authorization: Bearer {token}`
- On HTTP error: tries to parse JSON `detail` field, falls back to response text
- Converts response JSON to Python dict via `json.loads()`

---

### src/models/users_model.rs

**Role**: Authentication response models. Populated after `connect()`.

**`User`** (pyclass):

| Field | Type | API JSON key |
|---|---|---|
| `user_id` | `i32` | `userId` |
| `user_name` | `Option<String>` | `userName` |
| `first_name` | `Option<String>` | `firstName` |
| `middle_name` | `Option<String>` | `middleName` |
| `last_name` | `Option<String>` | `lastName` |
| `email` | `Option<String>` | `email` |
| `user_type_id` | `i32` | `userTypeId` |
| `org_id` | `i32` | `orgId` |
| `user_type_details` | `UserTypeDetails` | `userTypeDetails` |
| `org_details` | `OrgDetails` | `orgDetails` |
| `profile_pic` | `Option<String>` | `profilePic` |
| `token` | `Option<String>` | `token` |
| `token_expiry_date` | `Option<String>` | `tokenExpiryDate` |

**`UserTypeDetails`** (pyclass): `user_type_id: i32`, `user_type: Option<String>`

**`OrgDetails`** (pyclass): `org_id: i32`, `org_name: Option<String>`

Serde mapping: `rename_all = "camelCase"`

---

### src/models/login_models.rs

**Role**: Request body for `POST /session/new`.

**`LoginRequest`**: `login_id: String`, `passwd: String` → serialized as camelCase

**v2 `NewSession` schema** (additional optional fields accepted by the API):
- `ipAddress: String` (default `127.0.0.1`)
- `deviceInfo: String|null`

---

### src/models/datasets_model.rs

**Role**: All dataset-related data structures, both read (API responses) and write (request bodies).

**`Dataset`** (pyclass, immutable — API response):

| Field | Type | Notes |
|---|---|---|
| `dataset_id` | `i32` | |
| `dataset_name` | `String` | |
| `dataset_type` | `i32` | 1=Vision, 2=Text, 3=Audio, … |
| `dataset_type_interp` | `String` | Human-readable type |
| `dataset_short_info` | `String` | Description |
| `dataset_status` | `i32` | 1=Active |
| `dataset_status_interp` | `String` | |
| `dataset_tags` | `String` | Comma-separated |
| `publish_type` | `i32` | 0=Private, 1=Internal, 2=Public |
| `created_on` | `String` | ISO datetime |
| `modified_on` | `String` | |
| `version_no` | `Option<String>` | Latest version string |

**`DatasetVersionDetails`** (pyclass — API response):

| Field | Type |
|---|---|
| `id` | `i32` |
| `user_id` | `i32` |
| `dataset_id` | `i32` |
| `version_availability` | `i32` — 1=available |
| `version_no` | `String` — "1.0.0" |
| `version_remark` | `String` |
| `publish_type` | `i32` |
| `created_on` | `String` |
| `modified_on` | `String` |

**`DatasetDownloadDetails`** (pyclass — return value after download):

| Field | Type |
|---|---|
| `dataset_id` | `i32` |
| `version_no` | `String` |
| `data_path` | `String` — local cache directory |
| `download_status` | `bool` — true = success |

**`DatasetItem`** (pyclass — single sample):

| Field | Type |
|---|---|
| `entity_id` | `String` |
| `files` | `Option<Vec<ItemFile>>` |
| `annotations` | `Option<Vec<HashMap<String, AnnotationValue>>>` |

**`ItemFile`** (pyclass): `file_id: String`, `file_name: String`, `file: PathBuf`

**`AnnotationValue`** (untagged enum): `Str`, `Int(i32)`, `Float(f64)`, `Bool`, `VecString`, `VecInt`, `VecFloat`, `VecBool`, `Map<String, AnnotationValue>`

**`NewDataset`** (pyclass, mutable — create request):

| Field | Type | Default |
|---|---|---|
| `user_id` | `i32` | 0 (filled by client) |
| `dataset_name` | `String` | required |
| `dataset_type` | `i32` | required |
| `dataset_short_info` | `String` | required |
| `dataset_tags` | `String` | required |
| `dataset_verification_type` | `i32` | 1 (Automatic) |

Methods: `__new__(dataset_name, dataset_type, dataset_short_info, dataset_tags, user_id=0, dataset_verification_type=1)`, `to_api_json() -> PyResult<String>`

**`UpdateDatasetRequest`** (pyclass, mutable — update request):
Fields (all optional): `dataset_name`, `dataset_status`, `remark`, `dataset_verification_type`

Methods: `__new__(...)`, `to_api_json() -> PyResult<String>`

**`NewDatasetEntity`** (pyclass, mutable — create entity request):

| Field | Type | Default |
|---|---|---|
| `dataset_id` | `i32` | 0 |
| `user_id` | `i32` | 0 |
| `ds_entity_name` | `String` | required |
| `entity_source` | `Option<String>` | None |
| `collected_on` | `String` | required (ISO datetime) |
| `labeling_algo` | `String` | required |
| `ds_entity_info` | `Py<PyAny>` | required (Python dict) |
| `location_id` | `Option<i32>` | None |
| `files_category` | `Option<Py<PyAny>>` | None (Python dict) |

Methods: `__new__(...)`, `to_api_json(py) -> PyResult<String>`, getters/setters for `ds_entity_info` and `files_category`

**`UpdateDatasetEntity`** (pyclass, mutable — update entity request):
All fields from `NewDatasetEntity` optional, plus:
- `ds_entity_status: Option<i32>`
- `remark: String` — required
- `version_id: i32` — 0 = latest
- `update_latest_entity: bool` — default false

Methods: `__new__(remark, ...)`, `to_api_json(py) -> PyResult<String>` — only includes non-None fields in output

**`DatasetLabel`** (pyclass, immutable — API response from `GET /datasets/labels/{dataset_id}`):

| Field | Type | Notes |
|---|---|---|
| `label_id` | `i32` | |
| `dataset_id` | `i32` | |
| `label` | `String` | |
| `created_on` | `Option<String>` | ISO datetime |
| `modified_on` | `Option<String>` | ISO datetime |

Derives `Serialize + Deserialize` with `rename_all = "camelCase"` for direct JSON round-trip.

**`UpdateDatasetLabel`** (pyclass, mutable — `PUT /datasets/labels/{dataset_id}`):
Fields: `label_id: i32`, `label: String`
Methods: `__new__(label_id, label)`, `to_api_json() -> PyResult<String>`

**`DeleteDatasetEntities`** (pyclass, mutable — `DELETE /datasets/datasetEntities`):
Fields: `dataset_entity_ids: Vec<String>`, `remark: String`, `version_id: Option<i32>`
Methods: `__new__(dataset_entity_ids, remark, version_id=None)`, `to_api_json() -> PyResult<String>`
Serde: `skip_serializing_if = "Option::is_none"` on `version_id`

**`NewDatasetVersion`** (pyclass, mutable — `POST /datasets/versions/new/{dataset_id}`):
Fields: `version_availability: i32` (default 1), `version_type: Option<String>`, `version_remark: Option<String>`
Methods: `__new__(version_availability=1, version_type=None, version_remark=None)`, `to_api_json() -> PyResult<String>`
Serde: `skip_serializing_if = "Option::is_none"` on optional fields

---

### src/models/benchmarks_model.rs

**Role**: Benchmark result data structures.

**`Benchmark`** (pyclass):

| Field | Type | Notes |
|---|---|---|
| `benchmark_id` | `String` | |
| `model_id` | `Option<String>` | JSON key: `mlModelId` |
| `dataset_id` | `i32` | |
| `dataset_version_id` | `i32` | |
| `model_version_id` | `Option<i32>` | |
| `benchmark_description` | `Option<String>` | |
| `benchmark_status` | `i32` | 0=pending, 1=running, 2=completed |
| `user_id` | `i32` | |
| `report_id` | `Option<i32>` | |
| `created_on` | `Option<String>` | |
| `modified_on` | `Option<String>` | |

**`MetricValue`** (untagged enum): same variants as `AnnotationValue`

**`Prediction`** (pyclass): `entity_id: String`, `original: Option<serde_json::Value>`, `predicted: Option<serde_json::Value>`

**`Results`** (pyclass): `metrics: Option<HashMap<String, MetricValue>>`, `predictions: Vec<Prediction>`

**`FileInformation`** (pyclass): `file_name`, `file_type` (MIME), `file_size: u64` (bytes), `file_hash`

**`BenchmarkResult`** (pyclass): `benchmark_id`, `model_id: Option<i32>`, `model_information: Option<Vec<FileInformation>>`, `file_information: Option<Vec<FileInformation>>`, `results: Results`

---

### src/datasets/datasets.rs

**Role**: Static methods implementing all dataset-related API calls. Called by `KappaApkClient` methods.

**Struct**: `Datasets` (no fields — namespace only)

**Methods**:

| Method | Endpoint | Description |
|---|---|---|
| `list_datasets_json(client, page?, size?, order_by?, order_keyword?)` | `GET /datasets/filter?page=&size=&orderBy=&orderKeyword=` | Raw paginated JSON |
| `list_datasets(client, ...)` | Same | Returns `List[Dataset]` pyclass objects |
| `get_dataset_details(client, dataset_id?, dataset_name?)` | `GET /datasets/filter?datasetId=X` or `?datasetName=Y` | First item from paginated response |
| `get_dataset_version_details(client, dataset_id?, dataset_name?, version_id?, version_no?)` | POST or GET (see below) | `DatasetVersionDetails` |
| `download_dataset_version_archive(client, ...)` | `GET /datasets/versions/archive/{dataset_id}/{dataset_version_no}` | Downloads ZIP, extracts, returns `DatasetDownloadDetails` |
| `load_kappa_dataset(client, ..., transform?, target_transform?, transform_input_mode?)` | — | Calls download, parses metadata, returns `KappaDataset` |
| `get_dataset_loader(client, ..., loader_type?, batch_size?, shuffle?, drop_last?, ...)` | — | Returns loader for "kappa"/"pytorch"/"transformers"/"tensorflow" |
| `add_dataset(client, body: String)` | `POST /datasets/new` | Creates dataset |
| `update_dataset(client, dataset_id, body)` | `PUT /datasets/{dataset_id}` | Updates dataset |
| `add_dataset_entity(client, dataset_id, entity_json, file_paths)` | `POST /datasets/datasetEntities/new/{dataset_id}` | Multipart entity+files |
| `update_dataset_entity(client, dataset_id, entity_id, update_json, file_paths)` | `PUT /datasets/datasetEntities/{dataset_id}/{dataset_entity_id}` | Multipart entity update |
| `filter_datasets(client, search?, dataset_id?, dataset_name?, dataset_type?, dataset_tags?, dataset_status?, publish_type?, page?, size?, order_by?, order_keyword?)` | `GET /datasets/filter` | Rich filter — tags, type, status, publish_type; defaults: page=1, size=20, order=modifiedOn DESC |
| `get_dataset_fields(client, dataset_id)` | `GET /datasets/fields/{dataset_id}` | Dataset input-field schema |
| `delete_dataset(client, dataset_id, remark?)` | `PUT /datasets/{dataset_id}` with `{datasetStatus:0}` | Soft-delete (service has no hard-delete) |
| `add_dataset_labels(client, dataset_id, labels)` | `POST /datasets/labels/{dataset_id}` | Adds label string list |
| `get_dataset_labels(client, dataset_id)` | `GET /datasets/labels/{dataset_id}` | Returns `Vec<DatasetLabel>` (typed) |
| `update_dataset_label(client, dataset_id, label_id, label)` | `PUT /datasets/labels/{dataset_id}` | Rename label by ID |
| `list_dataset_entities(client, dataset_id, version_id?)` | `GET /datasets/datasetEntities/{dataset_id}` | All entities in a version |
| `get_dataset_entity(client, dataset_id, entity_id, version_id?)` | `GET /datasets/datasetEntities/{dataset_id}/{entity_id}` | Single entity |
| `filter_dataset_entities(client, dataset_id, entity_name?, entity_status?, version_id?, page?, size?, order_by?, order?)` | `GET /datasets/datasetEntities/filter/{dataset_id}` | Paginated entity search; defaults: page=0, size=10, order=modifiedOn DESC |
| `delete_dataset_entities(client, dataset_entity_ids, remark, version_id?)` | `DELETE /datasets/datasetEntities` | Bulk soft-delete by ID list |
| `create_dataset_version(client, dataset_id, body_json)` | `POST /datasets/versions/new/{dataset_id}` | New version from `NewDatasetVersion` JSON |
| `list_dataset_versions(client, dataset_id, version_availability?)` | `GET /datasets/versions/{dataset_id}` | All versions; filter by availability |
| `delete_dataset_version(client, dataset_id, version_no)` | `DELETE /datasets/versions/{dataset_id}/{version_no}` | Delete one version |
| `publish_dataset_version(client, dataset_id, version_no, publish_type)` | `POST /datasets/versions/publish/{dataset_id}/{version_no}?publish_type=N` | 0=Private, 1=Internal, 2=Public |

**`get_dataset_version_details` resolution logic**:
- If `version_id` given → POST `/datasets/versions/details/list` with `[version_id]` array
- If `version_no` given → GET `/datasets/versions/{dataset_id}/{dataset_version_no}`
- Must resolve `dataset_id` first if only `dataset_name` provided

**`download_dataset_version_archive` behavior**:
1. Resolve dataset+version identifiers
2. Cache dir: `~/cache/kappa-framework/datasets/{dataset_name}_{version_no}/` (or custom `dataset_path`)
3. Return immediately if cache dir exists (no re-download)
4. Stream download to temp file (chunked)
5. Extract ZIP with zip-slip validation
6. Delete temp ZIP
7. Return `DatasetDownloadDetails`

**`get_dataset_loader` loader_type options**:
- `"kappa"` → `KappaDataLoader` (Rust, map-style, epoch-aware)
- `"pytorch"` → `torch.utils.data.DataLoader` wrapper
- `"transformers"` → HuggingFace Datasets integration
- `"tensorflow"` → TensorFlow Dataset API (needs `tf_output_signature`)

---

### src/datasets/kappa_dataloader.rs

**Role**: PyTorch-compatible data loader. Implements Python iterator protocol.

**Struct**: `KappaDataLoader`

| Field | Type | Default |
|---|---|---|
| `dataset` | `Py<KappaDataset>` | required |
| `batch_size` | `usize` | 32 |
| `shuffle` | `bool` | true |
| `drop_last` | `bool` | false |
| `indices` | `Vec<usize>` | shuffled epoch permutation |
| `position` | `usize` | current read position |

**Epoch semantics**: Every `__iter__()` call resets `position=0` and re-shuffles `indices`.

| Method | Returns | Description |
|---|---|---|
| `__new__(dataset, batch_size=32, shuffle=true, drop_last=false)` | `Self` | Init, does NOT start iteration |
| `__iter__(&mut self)` | `Self` | Resets epoch (reshuffle + reset position) |
| `__len__(&self)` | `usize` | Number of batches (uses filtered count when dropout active) |
| `__next__(&mut self)` | `Option<PyObject>` | Next batch as `List[Dict]`; `None` = epoch done |

**Dropout / entity exclusion** (mainline hooks; orchestrated by `Feedbacks` on `dataset-feedback-code-backup`):

| Method | Description |
|---|---|
| `set_dropout_enabled(enabled)` | When true, `set_excluded_entity_ids` is applied on next `__iter__` |
| `set_excluded_entity_ids(ids)` | Entity IDs to skip on next epoch |
| `clear_excluded_entity_ids()` | Reset exclusions |
| `num_active_samples()` | Sample count after last epoch reset |
| `excluded_count()` | Number of samples excluded |

If dropout would leave fewer than `min_remaining_entities` (default 1), the loader keeps the full dataset and emits a Python `warnings.warn`.

Batch item format: `Dict` with all sample fields + `entity_id` key.

Use with `Feedbacks.trace(..., batch=batch)` on branch `dataset-feedback-code-backup` so entity IDs are extracted automatically (no manual list).

---

# Feedbacks — Implementation Analysis & Research Review

> **Source branch:** `dataset-feedback-code-backup` — files below are not on mainline `v2.0.0` until merged.
>
> **Glossary:** [§0 — Abbreviations & Glossary](FEEDBACKS_RESEARCH.md#0-abbreviations--glossary) in `FEEDBACKS_RESEARCH.md` defines CSL, BLESS, GMM, EMA, and other acronyms used across Feedbacks docs.

### src/datasets/feedbacks.rs

**Role**: Training loss tracker that generates "bad sample" reports for model feedback.  
**Related**: `src/datasets/feedbacks_threshold.rs` (threshold / EMA / GMM).  
**Literature map**: [§10 — Algorithm & Literature Reference](#10-feedbacks--algorithm--literature-reference).  
**Research doc**: `docs/FEEDBACKS_RESEARCH.md` (§12 implementation, §5–§11 survey).

### src/datasets/feedbacks_threshold.rs

**Role**: Report-time analysis — epoch window, loss threshold modes, EMA smoothing, 2-component GMM.  
**Literature map**: [§10](#10-feedbacks--algorithm--literature-reference).

**Struct**: `Feedbacks`

| Field | Type | Default |
|---|---|---|
| `dataset_loader` | `Option<Py<PyAny>>` | None (reserved; not used by trace logic) |
| `loss_threshold` | `f32` | 0.7 — meaning depends on `loss_threshold_mode` |
| `loss_threshold_mode` | `str` | `absolute` \| `percentile` \| `relative` \| `gmm` |
| `epoch_threshold` | `f32` | 0.5 — fraction or fixed epoch count (see mode) |
| `epoch_threshold_mode` | `str` | `fraction` \| `fixed_epochs` |
| `smoothing_mode` | `str` | `none` \| `ema` |
| `ema_alpha` | `f32` | 0.3 |
| `traces` | `Vec<TraceEntry>` | empty |
| `last_loss` | `f32` | 0.0 |
| `last_epoch` | `i32` | 0 |
| `last_batch_idx` | `i32` | 0 |
| `report_path` | `Option<String>` | None |
| `report_submitted` | `bool` | false |

**`TraceEntry`** (internal): `epoch: i32`, `batch_idx: i32`, `loss: f32`, `entity_ids: Option<Vec<String>>`

**Entity ID extraction**: `extract_entity_ids_from_batch()` reads `entity_id` from each item in a `KappaDataLoader` batch (`list[dict]` or objects with `.entity_id`).

**Training enhancement — dropout bad samples:** exclude entities flagged in the **last two epochs** from `KappaDataLoader` before the next epoch. See `docs/FEEDBACKS_RESEARCH.md` [§11](FEEDBACKS_RESEARCH.md#11-training-enhancement--dropout-bad-samples-planning).

| Method | Description |
|---|---|
| `__new__(..., loss_threshold_mode="absolute", epoch_threshold_mode="fraction", smoothing_mode="none", ema_alpha=0.3)` | Initialize — see `docs/FEEDBACKS_RESEARCH.md` §12 |
| `trace(epoch, loss, batch_idx?, batch?, losses?)` | Record step; with `batch` from Kappa loader, entity IDs auto-extracted. `losses` = per-sample list aligned with entities. |
| `trace_with_entities(..., losses?)` | Explicit IDs and/or `batch` (`batch` wins if both set); `losses` for per-sample loss |
| `attribution_method` / `bless_k_max` | BLESS: `none` \| `frequency` \| `lasso` \| `bayesian`; enforces `\|entity_ids\| ≤ bless_k_max` when enabled |
| `set_model_summary(summary)` | Dict → `model_summary` in report (model name, framework, task, etc.) |
| Automatic `model_summary` | **Planned** — Tier 0 baseline + optional MLflow/W&B adapters; see `docs/FEEDBACKS_RESEARCH.md` §14 |
| `set_submit_context(dataset_id, ...)` | Metadata for server POST (`datasetVersionId`, `runLabel`, `mlModelId`, …) |
| `create_report_dict(py)` | Build report Python dict (see report structure below) |
| `format_report(style="summary", max_bad_entities=10, ...)` | Human-readable or truncated pretty JSON string |
| `print_report(...)` | Print formatted report to stdout |
| `save_report(report_path, pretty=True)` | Write JSON report (full lists); `pretty=False` for compact |
| `submit_report(py, client=None, dataset_id=None, ...)` | Offline: report dict. With `client`: POST `.../datasets/{id}/training-feedback` (schema v2; strips `report_path` / `report_submitted`) |
| `entities_to_drop(lookback_epochs?, rule?)` | Drop set for loader exclusion (§11); rules `any_hit` \| `csl_lite` |
| `apply_dropout_to_loader(loader, ...)` | Compute drop set + `loader.set_excluded_entity_ids` |
| `last_dropout_summary()` | Last dropout snapshot (also in report `dropout` section) |

**Report structure** (see `docs/FEEDBACKS_RESEARCH.md` §2 for full example):

| Key | Description |
|---|---|
| `model_summary` | From `set_model_summary()` |
| `threshold_config` | Modes, `loss_threshold_effective`, optional `gmm_details` |
| `training_summary` | Epoch window, `loss_stats`, trace metadata |
| `entity_summary` | `entities_tracked`, `entities_flagged`, `avg_loss_flagged`, **`worst`**, **`best_among_flagged`** |
| `bad_entities` | Ranked list: `entity_id`, `avg_loss`, `min_loss`, `max_loss`, `csl`, `epochs_flagged` |
| `entity_scores` | BLESS scores (`freq_score`, optional `lasso_score`, `bayes_score`, `ci_*`) when `attribution_method != none` |
| `attribution_meta` | BLESS run metadata (`method`, `k_max`, `baseline_loss`, `lasso_lambda`, …) |
| `bad_samples` | Batch-level flagged steps (`epoch`, `batch_idx`, `loss`, `entity_ids`) |
| `total_epochs`, `loss_stats`, … | Legacy flat fields (same values as inside `training_summary`) |

**"Bad sample" logic** (see `feedbacks_threshold.rs` + [§10](#10-feedbacks--algorithm--literature-reference)):

1. Resolve epoch window (`epoch_threshold_mode`: `fraction` | `fixed_epochs`).
2. Optionally apply EMA → `effective_loss` (`smoothing_mode`: `none` | `ema`).
3. Compute `loss_threshold_effective` from `loss_threshold_mode`: `absolute` | `percentile` | `relative` | `gmm`.
4. Flag trace if `effective_loss` exceeds threshold (GMM: posterior P(noisy) > 0.5).

```
total_epochs = max_epoch + 1
lookback_epochs = max(total_epochs * epoch_threshold, 1.0)   # fraction mode
min_epoch_considered = max(max_epoch - lookback_epochs + 1, 0)
```

### src/datasets/feedbacks_attribution.rs

**Role**: BLESS entity_scores (`freq_score`, LASSO, Bayesian EM). See `FEEDBACKS_RESEARCH.md` §9.7.

---

### src/datasets/dataloader_helper.rs

**Role**: Framework adapter utilities kept outside the core loader.

| Method | Description |
|---|---|
| `peek_batch(loader)` | `next(iter(loader))` — useful for TF signature inference |
| `infer_tf_output_signature(sample_batch)` | Builds nested `tf.TensorSpec` from one Kappa batch |

---

### src/benchmarks/benchmarks.rs

**Role**: Benchmark workflow object returned by `client.load_benchmark(id)`.

| Method | Description |
|---|---|
| `details()` | GET `/model-micro-services/v2/benchmarks/{id}` |
| `dataset(dataset_path?)` | Download + parse benchmark dataset (separate archive endpoint) |
| `setup_project()` | Random sample of project `src/` files → `file_information` hashes |
| `set_model_path(path)` | Hash model directory files |
| `save_benchmark(predictions, metrics)` | Build `BenchmarkResult` locally |
| `submit_benchmark()` | POST `/model-micro-services/v2/models/inferences/{model_id}` |

**Known gap:** `internal_save_benchmark` has TODO branches when neither model path nor prior `setup_project` data exists (`benchmarks.rs:449`).

---

### src/benchmarks/verifications.rs

**Role**: Standalone `BenchmarkVerification` pyclass — scans a local directory, hashes required files, POSTs to server for validation.

**Endpoints (v2 — public, no JWT per kappa-framework RBAC docs):**
- GET `.../v2/benchmarks/app/verifications/files/{benchmark_id}`
- POST `.../v2/benchmarks/app/verifications/{benchmark_id}`

Creates its own `reqwest::Client` and Tokio runtime per call.

---

### src/transforms/vision.rs

**Role**: Image transformation pipeline for computer vision datasets.

**`Compose`**: Stacks multiple transforms. Input can be:
- Sample dict (with `files` field — first file loaded as PIL image)
- PIL Image object
- File path string

**`ensure_pil_image(py, value)`**: Converts any of the above to PIL.Image (RGB). Detects PIL via `hasattr(value, "resize")`.

**`set_sample_image_if_dict(py, maybe_sample, image)`**: Sets `sample["image"]` if original input was a dict.

**Transform categories**: Resizing, Cropping, Normalization, Augmentation (exact functions available in source).

---

## 4. System Architecture & Design Patterns

### Pattern 1: Trait-Based Polymorphism (Rust)
`ApiClient` trait abstracts all HTTP operations. `KappaApkClient` is the concrete implementation. Enables mock clients for testing without network calls.

### Pattern 2: Python-Rust Bridge via PyO3
- Rust owns: networking, ZIP I/O, hashing, data loading
- Python owns: orchestration, ML frameworks, user code
- Boundary: JSON serialization (Rust `serde_json::Value` ↔ Python dict via `json.loads`)
- `json_value_to_pyobject()` is the core adapter

### Pattern 3: Blocking Adapter for Async Rust
- Tokio runtime created once per client instance
- `runtime.block_on(async_fn)` bridges async Rust → synchronous Python calls
- Avoids nested event loop issues

### Pattern 4: Filesystem Cache for Datasets
Downloaded dataset archives cached at `~/cache/kappa-framework/datasets/{name}_{version}/`. Second call with same params returns immediately without re-downloading.

---

## 5. Key Data Flows

### Authentication Flow
```
Python: client = KappaApkClient(base_url, login_id, passwd)
Python: client.connect()
  Rust: POST /session/new  ← LoginRequest {loginId, passwd}
  API:  ← {userId, token, userTypeId, orgDetails, ...}
  Rust: cache in login_response; extract token
Python: client.is_authenticated()  → True

All subsequent calls:
  Rust: add "Authorization: Bearer {token}" header automatically
```

### Dataset Download + Load Flow
```
Python: client.load_kappa_dataset(dataset_name="MyDS", version_no="1.0.0")
  Rust: get_dataset_details()
    GET /datasets/filter?datasetName=MyDS
    ← Dataset {dataset_id: 42, ...}
  Rust: get_dataset_version_details()
    GET /datasets/versions/42/1.0.0
    ← DatasetVersionDetails {id: 7, version_no: "1.0.0", ...}
  Rust: download_dataset_version_archive()
    check ~/cache/kappa-framework/datasets/MyDS_1.0.0/ → not found
    GET /datasets/versions/archive/42/1.0.0  (stream)
    write to temp file (chunked)
    extract ZIP → ~/cache/kappa-framework/datasets/MyDS_1.0.0/
    validate all paths (zip-slip prevention)
    delete temp ZIP
    ← DatasetDownloadDetails {data_path: "~/cache/.../MyDS_1.0.0/", download_status: true}
  Rust: parse metadata from extracted files
  Rust: build KappaDataset with DatasetItem list
← Py<KappaDataset>
```

### Training Loop with Feedbacks

Requires branch **`dataset-feedback-code-backup`** (or a merge that exports `Feedbacks`).

```
Python:
  loader = client.get_dataset_loader(..., loader_type="kappa", batch_size=32)
  fb = Feedbacks(loss_threshold=0.5, epoch_threshold=0.3)

  for epoch in range(10):
    for batch_idx, batch in enumerate(loader):
      loss = compute_loss(batch, model)
      fb.trace(epoch, loss.item(), batch_idx, batch=batch)

  fb.save_report("/reports/training_report.json")
  report = fb.submit_report()  # offline mode → returns dict
```

**Dropout training loop (Kappa loader):**

```python
fb = Feedbacks(loss_threshold=0.5, dropout_enabled=True, dropout_lookback_epochs=2)
loader.set_dropout_enabled(True)
for epoch in range(num_epochs):
    if epoch >= fb.dropout_lookback_epochs:
        fb.apply_dropout_to_loader(loader)
    for batch_idx, batch in enumerate(loader):
        fb.trace(epoch, loss.item(), batch_idx, batch=batch)
```

### Dataset Entity Upload Flow
```
Python:
  entity = NewDatasetEntity(
    ds_entity_name="sample_001",
    collected_on="2026-01-15T10:00:00",
    labeling_algo="manual",
    ds_entity_info={"label": "cat", "confidence": 0.95}
  )
  client.add_dataset_entity(42, entity, file_paths=["/data/img001.jpg"])

  Rust:
    read /data/img001.jpg → Vec<u8>
    entity.to_api_json(py) → camelCase JSON string
    multipart form:
      Part "entity": JSON string
      Part "files": [img001.jpg bytes]
    POST /datasets/datasetEntities/new/42
    ← {"msg": "Dataset item successfully added.", "data": {...}}
```

---

## 6. Public API Surface

### `KappaApkClient` (Python-exposed Rust class)

```python
# Construction
client = KappaApkClient(base_url: str, login_id: str, passwd: str)

# Authentication
client.connect() -> dict           # login
client.close() -> None             # logout
client.is_authenticated() -> bool
client.get_token() -> Optional[str]
client.get_base_url() -> str
client.set_base_url(url: str)

# Context manager
with KappaApkClient(...) as client:
    ...  # auto connect/close

# Dataset listing
client.list_datasets(page=1, size=200, order_by="datasetId", order_keyword="DESC") -> dict
client.list_datasets_typed(...) -> List[Dataset]

# Dataset version
client.get_dataset_version_details(
    dataset_id=None, dataset_name=None, version_id=None, version_no=None
) -> DatasetVersionDetails

# Download & load
client.download_dataset_version_archive(
    dataset_id=None, dataset_name=None, version_id=None, version_no=None, dataset_path=None
) -> DatasetDownloadDetails

client.load_kappa_dataset(
    dataset_id=None, dataset_name=None, version_id=None, version_no=None,
    dataset_path=None, transform=None, target_transform=None, transform_input_mode="content"
) -> KappaDataset

client.get_dataset_loader(
    ...,  # same as load_kappa_dataset
    loader_type="kappa",  # "kappa" | "pytorch" | "transformers" | "tensorflow"
    batch_size=32, shuffle=True, drop_last=False,
    tf_output_signature=None,
    transform=None, target_transform=None, transform_input_mode="content"
) -> DataLoader

# Dataset CRUD
client.add_dataset(dataset: NewDataset) -> dict
client.update_dataset(dataset_id: int, update: UpdateDatasetRequest) -> dict
client.delete_dataset(dataset_id: int, remark: str = None) -> dict        # soft-delete

client.add_dataset_entity(dataset_id: int, entity: NewDatasetEntity, file_paths: List[str] = []) -> dict
client.update_dataset_entity(dataset_id: int, entity_id: str, update: UpdateDatasetEntity, file_paths: List[str] = []) -> dict

# Dataset lookup & filtering
client.get_dataset_details(dataset_id=None, dataset_name=None) -> Dataset
client.filter_datasets(
    search=None, dataset_id=None, dataset_name=None, dataset_type=None,
    dataset_tags=None, dataset_status=None, publish_type=None,
    page=None, size=None, order_by=None, order_keyword=None
) -> dict
client.get_dataset_fields(dataset_id: int) -> dict

# Label management
client.add_dataset_labels(dataset_id: int, labels: List[str]) -> dict
client.get_dataset_labels(dataset_id: int) -> List[DatasetLabel]
client.update_dataset_label(dataset_id: int, label_id: int, label: str) -> dict

# Entity operations
client.list_dataset_entities(dataset_id: int, version_id: int = None) -> dict
client.get_dataset_entity(dataset_id: int, entity_id: str, version_id: int = None) -> dict
client.filter_dataset_entities(
    dataset_id: int, entity_name=None, entity_status=None, version_id=None,
    page=None, size=None, order_by=None, order=None
) -> dict
client.delete_dataset_entities(dataset_entity_ids: List[str], remark: str, version_id: int = None) -> dict

# Dataset version management
client.create_dataset_version(dataset_id: int, version: NewDatasetVersion) -> dict
client.list_dataset_versions(dataset_id: int, version_availability: int = None) -> dict
client.delete_dataset_version(dataset_id: int, version_no: str) -> dict
client.publish_dataset_version(dataset_id: int, version_no: str, publish_type: int) -> dict
# publish_type: 0=Private, 1=Internal, 2=Public

# Benchmarks
client.load_benchmark(benchmark_id: str) -> Benchmarks

# Benchmark verification (standalone — v2 public endpoints, no auth)
BenchmarkVerification(server_url, benchmark_id, path).result() -> bool

# Raw HTTP
client.make_request(method: str, endpoint: str, data=None, token=None) -> dict
```

### `Feedbacks` (Python-exposed Rust class)

> On branch **`dataset-feedback-code-backup`**. Not exported from mainline `v2.0.0` until merged.

```python
fb = Feedbacks(dataset_loader=None, loss_threshold=0.7, epoch_threshold=0.5)
fb.set_model_summary({"name": "my-model", "framework": "pytorch"})
# Kappa: pass the loader batch — entity IDs extracted in Rust
fb.trace(epoch: int, loss: float, batch_idx: int = 0, batch: list = None, losses: List[float] = None)
# Fallback: explicit IDs or batch (batch wins if both set)
fb.trace_with_entities(
    epoch: int, loss: float, batch_idx: int = 0,
    entity_ids: List[str] = None, batch: list = None,
)
fb.create_report_dict() -> dict
fb.format_report(style="summary"|"json", max_bad_entities=10, ...) -> str
fb.print_report(...)  # stdout
fb.save_report(report_path: str, pretty=True)
fb.set_submit_context(dataset_id: int, dataset_version_id=None, run_label=None, ...) -> None
fb.entities_to_drop(lookback_epochs=None, rule=None) -> list[str]
fb.apply_dropout_to_loader(loader: KappaDataLoader, ...) -> list[str]
fb.last_dropout_summary() -> dict
fb.submit_report(client=None, dataset_id=None, ...) -> dict
```

### `KappaDataLoader` (Python-exposed Rust class)

```python
loader = KappaDataLoader(dataset, batch_size=32, shuffle=True, drop_last=False)
loader.set_dropout_enabled(True)
loader.set_excluded_entity_ids(["id1", "id2"])
len(loader)          # number of batches
for batch in loader: # List[Dict] — auto-reshuffles each epoch
    ...
```

### `BenchmarkVerification`

```python
bv = BenchmarkVerification(server_url, benchmark_id, "/path/to/project")
ok = bv.result()  # POST file hashes to v2 verification endpoint
```

### Data Models

```python
# Create dataset
ds = NewDataset(
    dataset_name="MyDataset",
    dataset_type=1,           # 1=Vision
    dataset_short_info="...",
    dataset_tags="tag1,tag2",
    dataset_verification_type=1  # 1=Auto
)
ds.user_id = 123  # set by client automatically

# Update dataset
upd = UpdateDatasetRequest(
    dataset_name=None, dataset_status=None, remark=None, dataset_verification_type=None
)

# Create entity
entity = NewDatasetEntity(
    ds_entity_name="sample_001",
    collected_on="2026-01-15T10:00:00",
    labeling_algo="manual",
    ds_entity_info={"label": "cat"}
)

# Update entity
update = UpdateDatasetEntity(
    remark="Fixing label error",
    ds_entity_name=None,
    ds_entity_status=None,
    update_latest_entity=False
)
```

---

## 7. Configuration Reference

### Rust Client (runtime)

| Parameter | How to Set |
|---|---|
| `base_url` | Constructor arg or `set_base_url()` |
| `login_id` | Constructor arg |
| `passwd` | Constructor arg |
| `dataset_path` | Optional override for local cache directory |

---

## 8. Dependencies

### Rust (`Cargo.toml`)

| Crate | Version | Purpose |
|---|---|---|
| `pyo3` | 0.25 | Python bindings (pyclass, pyfunction, pymodule) |
| `reqwest` | 0.11 | HTTP client (rustls TLS, multipart, streaming) |
| `tokio` | 1 | Async runtime (full features) |
| `serde` | latest | Serialization framework |
| `serde_json` | latest | JSON serialization |
| `zip` | 0.6 | ZIP archive extraction |
| `dirs` | 5 | Cross-platform home/cache directories |
| `urlencoding` | 2.1 | URL percent-encoding |
| `rand` | latest | Shuffling (for KappaDataLoader) |
| `sha1` | latest | File hashing |

---

## 9. Notable Implementation Details

### v2 API — Current Standard (v1 is Legacy)
All new code must use v2 endpoints only. The v1 API is legacy and no longer supported. The v2 API (User Services v2.0.0, Dataset Services v2.0.0, Model Services v2.5.0) removes `{user_id}/{user_type_id}` from all path parameters — identity is now derived entirely from the JWT token server-side. Reference table of the v1→v2 migration (for legacy context only):

| Area | v1 pattern | v2 pattern |
|---|---|---|
| Session logout | `DELETE /session/{user_id}/{user_type_id}` | `DELETE /session/` |
| List datasets | `GET /datasets/filter/{uid}/{utid}` | `GET /datasets/filter` |
| Create dataset | `POST /datasets/new/{uid}/{utid}` | `POST /datasets/new` |
| Update dataset | `PUT /datasets/{uid}/{utid}/{dataset_id}` | `PUT /datasets/{dataset_id}` |
| Create entity | `POST /datasets/datasetEntities/new/{uid}/{utid}/{did}` | `POST /datasets/datasetEntities/new/{dataset_id}` |
| Update entity | `PUT /datasets/datasetEntities/{uid}/{utid}/{did}/{eid}` | `PUT /datasets/datasetEntities/{dataset_id}/{dataset_entity_id}` |
| Version lookup | `GET /datasets/versions/{uid}/{utid}/{did}/{ver}` | `GET /datasets/versions/{dataset_id}/{dataset_version_no}` |
| Archive download | `GET /datasets/versions/archive/{uid}/{utid}/{did}/{ver}` | `GET /datasets/versions/archive/{dataset_id}/{dataset_version_no}` |
| Version batch lookup | `POST /datasets/versions/details/list/{uid}/{utid}` | `POST /datasets/versions/details/list` |

### ZIP Security — Zip-Slip Prevention
`datasets.rs::download_dataset_version_archive` validates every extracted path against the target directory before writing. Uses `outpath.starts_with(&data_dir)`. Rejects archives that attempt directory traversal.

### Streaming Download — Memory Efficiency
Uses `reqwest` byte streaming (`bytes_stream()`) with `tokio::io::AsyncWriteExt`. Writes directly to disk in chunks; never buffers the entire archive in memory.

### Multipart Upload Structure
```
Content-Type: multipart/form-data
  Part "entity": application/json (camelCase JSON string)
  Part "files":  application/octet-stream (filename preserved)
  Part "files":  application/octet-stream (multiple files)
```

### HTTP Error Handling
`make_request` on non-2xx status:
1. Try parse response body as JSON
2. If JSON has `detail` field → use as error message
3. Else → use full response text as error
4. Raise `PyValueError`

### Pagination Defaults
`list_datasets`: page=1, size=200, orderBy=`datasetId`, orderKeyword=`DESC`
Response shape: `{"items": [...], "total": N, "page": P, "size": S, "pages": P}`

### camelCase ↔ snake_case Mapping
All Rust models use `#[serde(rename_all = "camelCase")]`. API sends/receives camelCase. Rust fields and Python properties use snake_case. The `json_value_to_pyobject` function preserves the original JSON keys (camelCase) when converting to Python dicts from raw API responses.

### Dataset Cache Location
Default: `{home_dir}/cache/kappa-framework/datasets/{dataset_name}_{version_no}/`

Override: `dataset_path` parameter in download/load methods.

Cache hit detection: if the directory exists, skip all network calls and return immediately.

### DatasetVersionDetails Lookup Priority
1. If `version_id` provided → POST with ID array (batch lookup)
2. If `version_no` provided → GET with version string (single lookup)
3. Must know `dataset_id` before calling either (resolved from `dataset_name` if needed)

### Transform Input Modes
`transform_input_mode` parameter on `load_kappa_dataset`:
- `"content"` (default): Transform receives the file content (bytes or PIL Image)
- `"path"`: Transform receives the file path string

### Feedbacks — Epoch Window Calculation
```
max_epoch = last recorded epoch (0-based)
total_epochs = max_epoch + 1
lookback_epochs = max(total_epochs * epoch_threshold, 1.0)
min_epoch_considered = max(max_epoch - lookback_epochs + 1, 0)
# epoch_threshold=0.5, max_epoch=9 → lookback=5 → min_epoch=5 (epochs 5–9)
# A trace at epoch=3 with max_epoch=9 → NOT included in bad sample analysis
```

### `UpdateDatasetEntity.to_api_json` — Conditional Serialization
Only includes fields that are non-None in the output JSON. `remark` is always included (required). `version_id=0` means "target latest version". `update_latest_entity=true` backfills changes to the latest entity record.

### Soft-Delete Pattern (Datasets)
The dataset service has **no hard-delete endpoint**. Deletion is implemented as a `PUT /datasets/{dataset_id}` with body `{"datasetStatus": 0, "remark": "..."}`. The server provides a `/datasets/recover` endpoint to undo this. The `delete_dataset` method in both `Datasets` (Rust) and `KappaApkClient` (Python) follow this pattern. Do not add a DELETE method for datasets.

### Dataset Label API
Labels are managed per-dataset via `/data-micro-services/v2/datasets/labels/{dataset_id}`:
- `POST` — add list of label strings
- `GET` — returns `Vec<DatasetLabel>` (typed, not raw JSON)
- `PUT` — rename an existing label by `labelId`

There is no label delete endpoint; use `datasetStatus` or the entity-level approach to remove labels.

### Benchmark Dataset Download
The `Benchmarks.dataset()` method uses `GET /model-micro-services/v2/benchmarks/datasets/download/{benchmark_id}` (model microservice), **not** the general dataset archive endpoint. Archive is cached at `~/cache/kappa-framework/benchmarks/{benchmark_id}/`. The same streaming + zip-slip-safe extraction logic from `Datasets::download_dataset_version_archive` is replicated there.

### `filter_datasets` vs `list_datasets`
`list_datasets` / `list_datasets_json` build their own `GET /datasets/filter` query with defaults **page=1, size=200, orderBy=datasetId**. `filter_datasets` is a separate method with richer filters and defaults **page=1, size=20, orderBy=modifiedOn**. They hit the same endpoint but are **not** thin wrappers of each other — pagination defaults differ.

### `filter_dataset_entities` pagination
Defaults: **page=0**, size=10 (differs from dataset list pagination which starts at page=1). Confirm against server contract before changing.

### Feedbacks — branch & merge status

| Branch | `Feedbacks` in `lib.rs` | `feedbacks*.rs` sources |
|---|---|---|
| `v2.0.0` (mainline) | No | Not present — use `KappaDataLoader` dropout hooks only |
| `dataset-feedback-code-backup` | Yes (`m.add_class::<Feedbacks>()`) | `feedbacks.rs`, `feedbacks_threshold.rs`, `feedbacks_attribution.rs` |

Merge checklist: register module in `datasets/mod.rs`, export in `lib.rs`, add `Feedbacks` to `kappa_apk.pyi`, run `./build_wheel.sh`.

---

## 10. Feedbacks — Algorithm & Literature Reference

> **Source branch:** `dataset-feedback-code-backup`. Algorithm map matches implemented code on that branch; mainline `v2.0.0` has loader hooks only until merge.
>
> **Glossary:** [§0](FEEDBACKS_RESEARCH.md#0-abbreviations--glossary).

Canonical map from **implemented code** (on `dataset-feedback-code-backup`) → **algorithm** → **paper**. When adding or changing logic in `feedbacks.rs` / `feedbacks_threshold.rs`, update this table and `docs/FEEDBACKS_RESEARCH.md` §12.9.

Extended survey (BLESS, group testing, SBFL, etc.): `docs/FEEDBACKS_RESEARCH.md` §5–§11.

### 10.1 Implemented (on `dataset-feedback-code-backup`)

| Code symbol / API | Algorithm | Paper title | Authors | Venue | Link |
|---|---|---|---|---|---|
| `loss_threshold_mode="absolute"` | High-loss cutoff (small-loss = clean heuristic) | *Generalized Cross Entropy Loss for Training Deep Neural Networks with Noisy Labels* | Bianca Zhang, Murat Sabuncu | NeurIPS 2018 | https://arxiv.org/abs/1805.07836 |
| (same heuristic family) | Co-teaching: exchange small-loss batches | *Co-teaching: Robust Training of Deep Neural Networks with Extremely Noisy Labels* | Bo Han, Quanming Yao, Xingrui Yu, Gang Niu, Miao Xu, Weihua Hu, Ivor Tsallis, Masashi Sugiyama | NeurIPS 2018 | https://arxiv.org/abs/1804.06872 |
| `loss_threshold_mode="percentile"` | Top-p quantile threshold in epoch window | (standard robust ranking; no single canonical paper) | — | — | — |
| `loss_threshold_mode="relative"` | `θ = median + k × MAD` | *The Influence Curve and Its Role in Robust Estimation* | Frank R. Hampel | JASA 1974 | https://www.tandfonline.com/doi/abs/10.1080/01621459.1974.10482962 |
| `loss_threshold_mode="gmm"` | 2-component Gaussian mixture + EM; flag high-mean component | *DivideMix: Learning with Noisy Labels as Semi-Supervised Learning* | Junnan Li, Richard Socher, Steven C. Hoi | ICLR 2020 | https://arxiv.org/abs/2002.07394 |
| `fit_gmm_2()`, `gmm_posterior_noisy()` | DivideMix-style clean/noisy split on loss distribution | (same as above) | Li, Socher, Hoi | ICLR 2020 | https://arxiv.org/abs/2002.07394 |
| `smoothing_mode="ema"`, `apply_ema_in_order()` | Dynamic Instance Hardness (EMA of loss) | *Curriculum Learning by Dynamic Instance Hardness* | Tianyi Zhou, Shengjie Wang, Jeffrey A. Bilmes | NeurIPS 2020 | https://proceedings.neurips.cc/paper/2020/hash/62000dee5a05a6a71de3a6127a68778a-Abstract.html |
| `epoch_threshold_mode="fraction"` | Trailing fraction of epochs for analysis | *Learning from Noisy Labels via Dynamic Loss Thresholding* (epoch dynamics) | Hao Yang, Youzhi Jin, Ziyin Li, Deng-Bao Wang, Lei Miao, Xin Geng, Min-Ling Zhang | arXiv 2021 | https://arxiv.org/abs/2104.02570 |
| `epoch_threshold_mode="fixed_epochs"` | Last N complete training cycles | *An Empirical Study of Example Forgetting…* (epoch-wise signal) | Mariya Toneva, Alessandro Sordoni, Remi Tachet des Combes, Adam Trischler, Yoshua Bengio, Geoffrey J. Gordon | ICLR 2019 | https://arxiv.org/abs/1812.05159 |
| `build_entity_sections()`, `csl` field | Cumulative / repeated high-loss exposure (batch-level CSL proxy) | *Loss Knows Best: Detecting Annotation Errors in Videos via Loss Trajectories* | Praditha Alwis, Soumyadeep Chandra, Deepak Ravikumar, Kaushik Roy | arXiv 2026 (ICML track) | https://arxiv.org/abs/2602.15154 |
| `trace(..., batch=batch)` | Per-entity attribution via `entity_id` in batch | (Kappa dataset schema; enables CSL-style aggregation) | — | — | — |
| `attribution_method="frequency"`, `laplace_freq` | *Loss Knows Best* (CSL proxy); group-testing freq | Alwis et al.; Chan et al. | arXiv 2026; Allerton 2011 | https://arxiv.org/abs/2602.15154 ; https://arxiv.org/abs/1202.0206 |
| `attribution_method="lasso"`, `lasso_coordinate_descent` | Basis Pursuit / sparse recovery; EL2N-style scoring | Candès, Romberg & Tao; Paul et al. | IEEE TIT 2006; NeurIPS 2021 | https://arxiv.org/abs/math/0409186 ; https://arxiv.org/abs/2107.07075 |
| `attribution_method="bayesian"`, `bayesian_em` | Co-teaching responsibility; DivideMix noisy split | Han et al.; Li, Socher & Hoi | NeurIPS 2018; ICLR 2020 | https://arxiv.org/abs/1804.06872 ; https://arxiv.org/abs/2002.07394 |
| `bless_k_max` (batch ≤ 16) | Small-batch attribution constraint | Zhou, Wang & Bilmes | NeurIPS 2020 | https://proceedings.neurips.cc/paper/2020/hash/62000dee5a05a6a71de3a6127a68778a-Abstract.html |

**GMM fallback:** If &lt; 10 losses in window → `percentile` mode (`GMM_MIN_SAMPLES` in `feedbacks_threshold.rs`).

**EMA scope:** Batch-trace order when `losses` omitted; per-entity timeline when `losses` provided on `trace` / `trace_with_entities`.

### 10.2 Planned (documented, not in code yet)

| Planned feature | Paper title | Authors | Venue | Link |
|---|---|---|---|---|
| Dropout bad samples in loader | *Learning with Bad Training Data via Iterative Trimmed Loss Minimization* | Yanyao Shen, Sujay Sanghavi | ICML 2019 | https://arxiv.org/abs/1810.11874 |
| Dropout (small-loss selection) | *Co-teaching…* (see above) | Han et al. | NeurIPS 2018 | https://arxiv.org/abs/1804.06872 |
| BLESS freq_score → Ochiai (future) | *An Evaluation of Similarity Coefficients for Software Fault Localization* | Nuno R. Abreu, Peter Zoeteweij, Arjan J.C. van Gemund | PRDC 2006 / JSS 2009 | https://ieeexplore.ieee.org/document/4041886 |
| Forgetting-event counter | *An Empirical Study of Example Forgetting…* | Toneva et al. | ICLR 2019 | https://arxiv.org/abs/1812.05159 |
| Per-sample loss + true CSL | *DivideMix…*; *Loss Knows Best…* | Li et al.; Alwis et al. | ICLR 2020; arXiv 2026 | links above |
| Influence-based scoring | *Estimating Training Data Influence by Tracing Gradient Descent (TracIn)* | Garima Pruthi, Frederick Liu, Satyen Kale, Aniruddh Sundararajan | NeurIPS 2020 | https://arxiv.org/abs/2002.08484 |

### 10.3 Code file index

| File | Branch | Functions | Primary papers |
|---|---|---|---|
| `feedbacks_threshold.rs` | `dataset-feedback-code-backup` | `resolve_epoch_window`, `apply_ema_in_order`, `resolve_loss_threshold`, `fit_gmm_2`, `is_trace_flagged` | DivideMix (GMM); DIH (EMA); Hampel (relative); DLT (epoch window) |
| `feedbacks.rs` | `dataset-feedback-code-backup` | `trace`, `build_report_value`, `build_entity_sections`, `analyze_traces` | Loss Knows Best (CSL); all threshold modes via `feedbacks_threshold` |
| `feedbacks_attribution.rs` | `dataset-feedback-code-backup` | `build_bless_attribution`, LASSO, Bayesian EM | BLESS §9 — freq / lasso / bayesian |
| `kappa_dataloader.rs` | mainline + backup | `set_dropout_enabled`, `set_excluded_entity_ids` | Dropout integration (§11) |

### 10.4 Report JSON — literature-related fields

| Field | Source algorithm |
|---|---|
| `threshold_config.loss_threshold_effective` | Resolved θ from §10.1 loss mode |
| `threshold_config.gmm_details` | DivideMix-style GMM fit |
| `bad_samples[].loss_effective` | DIH / EMA when `smoothing_mode="ema"` |
| `bad_entities[].csl` | Loss Knows Best (flagged-batch count proxy) |
| `entity_summary.worst` / `best_among_flagged` | Ranked by effective loss (CSL aggregation) |

---

## 11. Code Review Backlog

Tracked in **[docs/CODE_REVIEW_BACKLOG.md](CODE_REVIEW_BACKLOG.md)** with per-item status (`open` / `done` / `deferred`).

Recent fixes (2026-07-01):

| ID | Status | Summary |
|---|---|---|
| CR-02 | done | v1 API removed — `BenchmarkVerification` → v2; `users.rs` → `GET /user-micro-services/v2/users/me` |
| CR-03 | done | Authenticated reads use `require_token()` instead of `get_token()` |
| CR-04 | done | Verification `result()` defaults to `false` when response lacks `valid`/`success` |
| CR-15 | done | v2 docstrings on `add_dataset` / `update_dataset` |
| CR-20 | done | `traits.rs` comment updated for JWT identity |

See the linked doc for the full checklist (P0–P3) and changelog.

---

## 12. Sync & Mapping Docs

| Document | Purpose |
|---|---|
| [SYNC_TRACKING.md](SYNC_TRACKING.md) | Which upstream tag is synced, commit hashes, file-level changelog, sync procedure |
| [FEATURE_CODE_MAPPING.md](FEATURE_CODE_MAPPING.md) | Legacy `kf_sdk` → `kappa_apk` method/model mapping, feature availability matrix |
| [README.md](README.md) | Index of all internal docs |

**Current state:** synced from `kappa-apk-3.0.0-beta` (`0a319ef`) on 2026-07-02.

---

*End of internal developer reference. For usage examples see `code_examples/`. For Feedbacks research depth see `docs/FEEDBACKS_RESEARCH.md`.*
