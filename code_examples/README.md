# Code Examples

This directory contains practical examples of how to use the kf-sdk Rust + Python library.

## Setup

```bash
./code_examples/setup_venv.sh
source code_examples/.venv/bin/activate
```

Upgrade after pulling repo changes:

```bash
./code_examples/upgrade_venv.sh
source code_examples/.venv/bin/activate
```

## Examples

### `mnist_kappa_training_example.py`

**Purpose**: Train a small CNN on **Fashion-MNIST** loaded from Kappa via `KappaDataLoader` (no Feedbacks).

**Requirements**: `torch`, `torchvision`, `matplotlib`, `kappa_apk`, local Kappa with Fashion-MNIST uploaded.

**Upload dataset first** (one-time):

```bash
source code_examples/.venv/bin/activate
python upload_torchvision_to_kappa_example.py --dataset fashion_mnist \
  --base-url http://127.0.0.1:8060 --login-id admin --password '***' \
  --limit-per-class 50
```

**Train**:

```bash
python mnist_kappa_training_example.py \
  --base-url http://127.0.0.1:8060 --login-id admin --password '***'

python mnist_kappa_training_example.py --epochs 3 --max-batches 20
```

Output: `code_examples/output/mnist_kappa_training_curves.png` and `mnist_kappa_training_metrics.json`

### `upload_torchvision_to_kappa_example.py`

**Purpose**: Export Fashion-MNIST (or CIFAR-100) from torchvision and upload to Kappa.

**Shared helpers**: `upload_kappa_helpers.py`

```bash
python upload_torchvision_to_kappa_example.py --dataset fashion_mnist \
  --limit-per-class 20 --export-only   # PNG export only, no API
```

### `http_client_authentication_example.py`

**Purpose**: Demonstrates HTTP client authentication and API calls.

```bash
./build_wheel.sh -i
python3 code_examples/http_client_authentication_example.py
```

### `bulk_upload.py`

**Purpose**: Async bulk entity upload (CSV ≤ 2 GB client / BE default 50 MB, or zip ≤ 50 GB) with transfer % and job polling (`BulkUploadJob`). Requires Kappa-framework **≥ 2.10.0**.

Archive `input_output` needs `dataset_schema.inputDataPath` (use `--input-data-path`).

```bash
source code_examples/.venv/bin/activate
export KAPPA_URL=http://127.0.0.1:8060
export KAPPA_USER=admin
export KAPPA_PASSWORD='***'
python code_examples/bulk_upload.py --dataset-id 42 --file ./data.zip \
  --upload-type archive --archive-layout input_output --input-data-path input
```

### `dataset_operations_example.py`

**Purpose**: End-to-end dataset scripting flow — create/lookup dataset, labels, single entity (`file_category` + `split`), filter, version, optional publish, RBAC check.

```bash
source code_examples/.venv/bin/activate
export KAPPA_URL=http://127.0.0.1:8060
export KAPPA_USER=admin
export KAPPA_PASSWORD='***'
python code_examples/dataset_operations_example.py \
  --dataset-name apk-demo-dataset --image ./sample.jpg
```

### `dataset_lifecycle_example.py`

**Purpose**: Lifecycle ops on an existing dataset — filter/list, custom schema, mark-labeled, soft-delete/recover entities (and optional dataset), version refresh/recover, entity file download.

```bash
source code_examples/.venv/bin/activate
export KAPPA_URL=http://127.0.0.1:8060
export KAPPA_USER=admin
export KAPPA_PASSWORD='***'
python code_examples/dataset_lifecycle_example.py --dataset-name apk-demo-dataset \
  --mark-labeled --soft-delete-entity --recover-entity
```

## Directory Structure

```
code_examples/
├── README.md
├── mnist_kappa_training_example.py      # CNN + KappaDataLoader (Fashion-MNIST)
├── upload_torchvision_to_kappa_example.py
├── upload_kappa_helpers.py
├── http_client_authentication_example.py
├── bulk_upload.py                       # Bulk upload + job progress (Kappa ≥ 2.10)
├── dataset_operations_example.py        # Dataset CUD → entities → version flow
├── dataset_lifecycle_example.py         # Schema, mark-labeled, soft-delete/recover
├── setup_venv.sh
├── upgrade_venv.sh
├── requirements.txt
└── output/                              # Generated artifacts (gitignored)
```
