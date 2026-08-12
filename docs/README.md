# kf-sdk documentation (`import kappa_apk`)

> **PyPI:** `kf-sdk` · **Import:** `kappa_apk` · **Version:** 3.0.4 · **Branch:** `main`  
> **Python:** 3.9+ · **API:** Kappa-framework **v2** (JWT identity — no `user_id` / `user_type_id` in URLs)  
> **Requires backend:** Kappa-framework **≥ 2.11.0** (`kappa_apk.min_backend_version()`)
>
> The PyPI name **`kf-sdk`** is kept for continuity with earlier releases; the import module is intentionally **`kappa_apk`**.

Python client for the [Kappa-framework](https://github.com/nsu-ai/kappa) ML platform. Built with Rust + [PyO3](https://pyo3.rs); HTTP calls go through the **Traefik API gateway** at `base_url`.

**Русская версия:** [docs/ru/README.md](ru/README.md) · **README:** [README.md](../README.md) (RU) · [README.en.md](../README.en.md) (EN)

---

## Documentation index

| Document | Description |
|---|---|
| [GettingStarted.md](GettingStarted.md) | Install, gateway setup, first script |
| [KappaApkClient.md](KappaApkClient.md) | Full client method reference + v2 HTTP paths |
| [DataModels.md](DataModels.md) | Request/response types (`Dataset`, `NewDataset`, …) |
| [Datasets.md](Datasets.md) | CRUD, labels, entities, **bulk upload / jobs**, versions, download & cache |
| [LoadersAndTransforms.md](LoadersAndTransforms.md) | `KappaDataset`, `KappaDataLoader`, vision/text/audio transforms |
| [Benchmarks.md](Benchmarks.md) | Benchmark workflow + `BenchmarkVerification` |
| [BuildAndPublish.md](BuildAndPublish.md) | Local wheels, cibuildwheel, PyPI / CI |
| [`code_examples/bulk_upload.py`](../code_examples/bulk_upload.py) | Bulk upload + transfer % + job progress |
| [`code_examples/dataset_operations_example.py`](../code_examples/dataset_operations_example.py) | Dataset CUD → labels → entity → version flow |
| [`code_examples/dataset_lifecycle_example.py`](../code_examples/dataset_lifecycle_example.py) | Schema, mark-labeled, soft-delete/recover |

---

## Quick example

```python
from kappa_apk import KappaApkClient

with KappaApkClient("https://kappa.nsu.ru:8061/", "user@example.com", "secret") as client:
    for batch in client.get_dataset_loader(dataset_name="MyDataset", version_no="1.0.0"):
        print(batch[0]["entity_id"])
```

---

## Package layout

```
kappa_apk/                  # native extension (import name)
├── KappaApkClient          # main HTTP client
├── KappaDataset            # in-memory dataset
├── KappaDataLoader         # Rust batch loader
├── Benchmarks              # via client.load_benchmark()
├── BenchmarkVerification   # file verification helper
├── Dataset, DatasetItem, … # typed models
├── Compose, Resize, …      # vision transforms (top-level exports)
├── BERTTokenizer, …        # text transforms
└── MelSpectrogram, …       # audio transforms
```

Type stubs: [`kappa_apk.pyi`](../kappa_apk.pyi) · Examples: [`code_examples/`](../code_examples/)
