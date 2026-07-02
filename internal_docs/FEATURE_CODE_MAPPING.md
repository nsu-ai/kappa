# Feature & Code Mapping

> Cross-reference between the **legacy OpenAPI `kf_sdk`** (removed), the **synced Rust `kappa_apk`** SDK, and the **private KappaApk source**.
> See [SYNC_TRACKING.md](SYNC_TRACKING.md) for which tag is currently synced.

---

## Package & Import Mapping

| Legacy (removed) | Current (synced) | Notes |
|---|---|---|
| `pip install kf-sdk` | `pip install kappa-apk` | Package renamed |
| `import kf_sdk` | `import kappa_apk` | Module renamed |
| `kf_sdk.ApiClient` | `kappa_apk.KappaApkClient` | Single high-level client |
| `kf_sdk.Configuration` | Constructor args `(base_url, login_id, passwd)` | No separate config object |
| `kf_sdk.SessionManagementApi` | `KappaApkClient.connect()` / context manager | Auth built into client |
| `kf_sdk.DatasetManagementApi` | `KappaApkClient.*` dataset methods | See API table below |
| Pydantic models in `kf_sdk.models.*` | `kappa_apk.Dataset`, `DatasetItem`, etc. | Rust-backed immutable types |
| — | `kappa_apk.KappaDataLoader` | **New** — epoch-aware batch loader |
| — | `kappa_apk.KappaDataset` | **New** — in-memory dataset container |
| — | `kappa_apk.BenchmarkVerification` | **New** — standalone v2 verification |
| — | `kappa_apk.vision`, `.text`, `.audio` | **New** — transform submodules |

---

## API Version Migration

| Area | Legacy `kf_sdk` | Current `kappa_apk` |
|---|---|---|
| Auth endpoint | `/user-micro-services/v1` | `/user-micro-services/v2` |
| Dataset endpoint | `/data-micro-services/v1` | `/data-micro-services/v2` |
| User identity | Explicit `user_id` / `user_type_id` in URL paths | Derived from JWT after login |
| Session creation | `SessionManagementApi.get_new_session(NewSession(...))` | `KappaApkClient(base_url, login, passwd)` + `connect()` |
| Anonymous session | `new_anonymous_session_session_anonymous_token_post()` | Not exposed in current SDK |

---

## Legacy API → New Client Method Mapping

### Session / Auth

| Legacy `kf_sdk` | New `kappa_apk` | HTTP (v2) |
|---|---|---|
| `SessionManagementApi.get_new_session(NewSession)` | `KappaApkClient(...).__enter__()` / `.connect()` | `POST /user-micro-services/v2/sessions` |
| `SessionManagementApi.new_anonymous_session_*` | — | Not implemented |
| Manual token management | `.get_token()`, `.is_authenticated()` | JWT cached in client |
| `ApiClient` context manager | `with KappaApkClient(...) as client:` | Auto connect/close |

### Dataset Versions

| Legacy `kf_sdk` | New `kappa_apk` | HTTP (v2) |
|---|---|---|
| `get_dataset_versions_*(user_id, user_type_id, dataset_id)` | `client.list_dataset_versions(dataset_id)` | `GET /data-micro-services/v2/datasets/{id}/versions` |
| `get_dataset_version_*(..., dataset_version_no)` | `client.get_dataset_version_details(dataset_id, version_no)` | `GET .../versions/{version_no}` |
| `get_dataset_version_archive_*(...)` | `client.download_dataset_version_archive(dataset_id, version_no)` | `GET .../versions/{version_no}/archive` |
| — | `client.load_kappa_dataset(dataset_name=..., version_no=...)` | Download + parse into `KappaDataset` |
| — | `client.get_dataset_loader(dataset_name=..., version_no=...)` | Download + return `KappaDataLoader` |

### Dataset CRUD

| Legacy `kf_sdk` | New `kappa_apk` | HTTP (v2) |
|---|---|---|
| — | `client.list_datasets(size=, page=)` | `GET /data-micro-services/v2/datasets` |
| — | `client.list_datasets_typed(...)` | Same, returns typed `Dataset` list |
| — | `client.get_dataset_details(dataset_id)` | `GET /data-micro-services/v2/datasets/{id}` |
| — | `client.filter_datasets(...)` | `POST /data-micro-services/v2/datasets/filter` |
| — | `client.add_dataset(NewDataset)` | `POST /data-micro-services/v2/datasets` |
| — | `client.update_dataset(dataset_id, UpdateDatasetRequest)` | `PUT /data-micro-services/v2/datasets/{id}` |
| — | `client.delete_dataset(dataset_id)` | `DELETE /data-micro-services/v2/datasets/{id}` |
| — | `client.get_dataset_fields(dataset_id)` | `GET .../datasets/{id}/fields` |

### Dataset Entities & Labels

| Legacy `kf_sdk` | New `kappa_apk` | HTTP (v2) |
|---|---|---|
| — | `client.list_dataset_entities(dataset_id, ...)` | `GET .../datasets/{id}/entities` |
| — | `client.get_dataset_entity(dataset_id, entity_id)` | `GET .../entities/{entity_id}` |
| — | `client.filter_dataset_entities(...)` | `POST .../entities/filter` |
| — | `client.add_dataset_entity(...)` | `POST .../entities` (multipart) |
| — | `client.update_dataset_entity(...)` | `PUT .../entities/{entity_id}` |
| — | `client.delete_dataset_entities(DeleteDatasetEntities)` | `POST .../entities/delete` |
| — | `client.add_dataset_labels(...)` | `POST .../labels` |
| — | `client.get_dataset_labels(dataset_id)` | `GET .../labels` |
| — | `client.get_dataset_label_names(dataset_id)` | Convenience wrapper |
| — | `client.update_dataset_label(...)` | `PUT .../labels/{label_id}` |

### Dataset Versioning

| Legacy `kf_sdk` | New `kappa_apk` | HTTP (v2) |
|---|---|---|
| — | `client.create_dataset_version(NewDatasetVersion)` | `POST .../versions` |
| — | `client.publish_dataset_version(dataset_id, version_no)` | `POST .../versions/{version_no}/publish` |
| — | `client.delete_dataset_version(dataset_id, version_no)` | `DELETE .../versions/{version_no}` |

### Benchmarks

| Legacy `kf_sdk` | New `kappa_apk` | HTTP (v2) |
|---|---|---|
| — | `client.load_benchmark(benchmark_id)` | Returns `Benchmarks` workflow object |
| — | `BenchmarkVerification(base_url).verify(...)` | Standalone, no JWT required |

---

## Rust Source File Mapping

Maps synced `src/` files to functional areas. Source of truth: private KappaApk at synced tag.

| File | Feature Area | Key Types / Functions |
|---|---|---|
| `src/lib.rs` | Python module registration | `kappa_apk` pymodule, `version()` |
| `src/client.rs` | HTTP client | `KappaApkClient` — all API methods |
| `src/traits.rs` | Client abstraction | `ApiClient` trait, `require_token()` |
| `src/models/datasets_model.rs` | Dataset types | `Dataset`, `DatasetItem`, `NewDataset`, etc. |
| `src/models/benchmarks_model.rs` | Benchmark types | `Benchmark`, `Results`, `Predictions` |
| `src/models/login_models.rs` | Auth request | `LoginRequest` |
| `src/models/users_model.rs` | User types | `User`, `OrgDetails` |
| `src/datasets/datasets.rs` | Dataset API layer | `Datasets`, `KappaDataset` |
| `src/datasets/kappa_dataloader.rs` | Data loading | `KappaDataLoader`, `filter_indices_for_dropout()` |
| `src/datasets/dataloader_helper.rs` | Loader utilities | `DataLoaderHelper.peek_batch()` |
| `src/datasets/dataset_entities.rs` | Entity helpers | **Empty stub** on 3.0.0-beta |
| `src/benchmarks/benchmarks.rs` | Benchmark workflow | `Benchmarks` (load, dataset, save, submit) |
| `src/benchmarks/verifications.rs` | File verification | `BenchmarkVerification` (v2, no auth) |
| `src/users/users.rs` | User profile | `get_user_profile()` → v2 `/users/me` |
| `src/transforms/vision.rs` | Image transforms | `Compose`, `Resize`, `Normalize`, etc. |
| `src/transforms/text.rs` | Text transforms | Tokenization helpers |
| `src/transforms/audio.rs` | Audio transforms | Spectrogram helpers |
| `src/utils/file_utils.rs` | File I/O | Path, hashing helpers |
| `src/utils/git_utils.rs` | Git blob hashing | File fingerprinting |
| `src/utils/python_json.rs` | PyObject ↔ JSON | Serde bridge |
| `src/utils/zip_utils.rs` | Zip extraction | Archive helpers |
| `src/utils/metric_utils.rs` | Metrics | Scoring helpers |

### Not in 3.0.0-beta (upstream branches)

| File | Branch | Feature |
|---|---|---|
| `src/datasets/feedbacks.rs` | `dataset-feedback-code-backup` | Training loss feedback tracker |
| `src/datasets/feedbacks_threshold.rs` | `dataset-feedback-code-backup` | Threshold / EMA / GMM modes |
| `src/datasets/feedbacks_attribution.rs` | `dataset-feedback-code-backup` | BLESS entity scoring |

---

## Data Model Mapping

| Legacy `kf_sdk.models` | New `kappa_apk` | Notes |
|---|---|---|
| `NewSession` | Constructor `(base_url, login_id, passwd)` | No model class |
| `UserDetails` | `User` (internal) | Not exported to Python on 3.0.0-beta |
| `UserTypeDetails` | `UserTypeDetails` (internal) | Not exported |
| `OrganizationDetails` | `OrgDetails` (internal) | Not exported |
| `DatasetDetails` | `Dataset` | Immutable, Rust-backed |
| `DatasetVersion` | `DatasetVersionDetails` | Version metadata |
| `ConfigurationDetails` | — | Removed (no config service in SDK) |
| `ValidationError` | — | Errors raised as Python exceptions |
| `HTTPValidationError` | — | Errors raised as Python exceptions |
| — | `DatasetItem` | Single labelled sample |
| — | `ItemFile` | File within a sample |
| — | `DatasetLabel` | Label record |
| — | `NewDataset` | Create dataset request |
| — | `UpdateDatasetRequest` | Update metadata |
| — | `NewDatasetEntity` | Create entity |
| — | `UpdateDatasetEntity` | Update entity |
| — | `DeleteDatasetEntities` | Bulk soft-delete |
| — | `NewDatasetVersion` | Create version |
| — | `UpdateDatasetLabel` | Rename label |
| — | `DatasetDownloadDetails` | Local cache after download |

---

## Feature Availability Matrix

| Feature | `kf_sdk` v1.0 | `kappa_apk` 3.0.0-beta | KappaApk branch |
|---|---|---|---|
| User login (v1 API) | Yes | No (v2 only) | — |
| Dataset version list/get/archive | Yes (v1, explicit user IDs) | Yes (v2, JWT) | — |
| Dataset CRUD | No | Yes | — |
| Entity CRUD | No | Yes | — |
| Label management | No | Yes | — |
| Version create/publish/delete | No | Yes | — |
| `KappaDataLoader` | No | Yes | — |
| Entity dropout in loader | No | Yes | `filter_indices_for_dropout()` |
| Image/text/audio transforms | No | Yes | — |
| Benchmarks workflow | No | Yes | v2 API |
| Benchmark file verification | No | Yes | No JWT |
| Feedbacks (loss tracking) | No | No | `dataset-feedback-code-backup` |
| Anonymous session | Yes | No | — |

---

## Documentation Mapping

| Legacy docs (removed) | Current docs (synced) |
|---|---|
| `docs/SessionManagementApi.md` | `docs/KappaApkClient.md` § Authentication |
| `docs/DatasetManagementApi.md` | `docs/Datasets.md` |
| `docs/DatasetDetails.md` | `docs/DataModels.md` |
| `docs/UserDetails.md` | — (user types not exported) |
| `docs/UsersManagementApi.md` | — |
| — | `docs/GettingStarted.md` |
| — | `docs/Benchmarks.md` |
| — | `docs/LoadersAndTransforms.md` |
| — | `docs/BuildAndPublish.md` |

---

*Last updated: 2026-07-02 | Synced tag: `kappa-apk-3.0.0-beta`*
