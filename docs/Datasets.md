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
    dataset_tags="vision,classification",
    dataset_type=1,
    dataset_status=1,
    publish_type=1,
    search="pizza",
    page=1,
    size=20,
)
```

---

## Create, update, delete

```python
from kappa_apk import NewDataset, UpdateDatasetRequest

result = client.add_dataset(NewDataset(
    dataset_name="MyDataset",
    dataset_type=1,
    dataset_short_info="Image classification",
    dataset_tags="vision",
    dataset_verification_type=1,
))

client.update_dataset(42, UpdateDatasetRequest(dataset_name="MyDataset-v2", remark="Renamed"))
client.delete_dataset(42, remark="Obsolete")   # soft-delete
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
page = client.filter_dataset_entities(42, entity_name="sample", page=0, size=20)

client.delete_dataset_entities(["uuid-1", "uuid-2"], remark="Duplicates removed")
```

---

## Versions

```python
from kappa_apk import NewDatasetVersion

client.create_dataset_version(42, NewDatasetVersion(version_availability=1, version_remark="Initial"))
versions = client.list_dataset_versions(42)
client.publish_dataset_version(42, "1.0.0", publish_type=2)
client.delete_dataset_version(42, "0.9.0")

details = client.get_dataset_version_details(dataset_name="MyDS", version_no="1.0.0")
```

---

## Download & cache

Archives download to `~/cache/kappa-framework/datasets/{dataset_name}_{version_no}/`. Repeat calls skip re-download if the cache directory exists.

```python
info = client.download_dataset_version_archive(dataset_name="MyDS", version_no="1.0.0")
print(info.data_path)
```

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
