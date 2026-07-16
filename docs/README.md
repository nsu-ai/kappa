# kf-sdk SDK documentation

> **Package:** `kf-sdk` (PyPI) · **Import:** `kappa_apk` · **Version:** 3.0.1 · **Branch:** `main`  
> **Python:** 3.9+ · **API:** Kappa-framework **v2** (JWT identity — no `user_id` / `user_type_id` in URLs)

Python client for the [Kappa-framework](https://github.com/nsu-ai/kappa) ML platform. Built with Rust + [PyO3](https://pyo3.rs); HTTP calls go through the **Traefik API gateway** at `base_url`.

**Русская версия:** [docs/ru/README.md](ru/README.md) · **README:** [README.md](../README.md) (RU) · [README.en.md](../README.en.md) (EN)

---

## Documentation index

| Document | Description |
|---|---|
| [GettingStarted.md](GettingStarted.md) | Install, gateway setup, first script |
| [KappaApkClient.md](KappaApkClient.md) | Full client method reference + v2 HTTP paths |
| [DataModels.md](DataModels.md) | Request/response types (`Dataset`, `NewDataset`, …) |
| [Datasets.md](Datasets.md) | CRUD, labels, entities, versions, download & cache |
| [LoadersAndTransforms.md](LoadersAndTransforms.md) | `KappaDataset`, `KappaDataLoader`, vision/text/audio transforms |
| [Benchmarks.md](Benchmarks.md) | Benchmark workflow + `BenchmarkVerification` |
| [BuildAndPublish.md](BuildAndPublish.md) | Local wheels, cibuildwheel, PyPI / CI |

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
