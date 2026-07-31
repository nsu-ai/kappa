# Датасеты

**English:** [../Datasets.md](../Datasets.md)

Руководство по операциям с датасетами через `kappa_apk`. Подробности HTTP-путей — в [KappaApkClient.md](KappaApkClient.md).

---

## Список и поиск

```python
# Постраничный JSON
page = client.list_datasets(page=1, size=50, order_by="modifiedOn", order_keyword="DESC")
items = page["items"]

# Типизированный список
datasets = client.list_datasets_typed()

# Расширенный фильтр
results = client.filter_datasets(
    dataset_tags="Image Classification",
    dataset_type=1,
    dataset_status=1,
    publish_type=1,
    search="pizza",
    page=1,
    size=20,
)
```

---

## Создание, обновление, удаление

```python
from kappa_apk import NewDataset, UpdateDatasetRequest, join_ml_tags

result = client.add_dataset(NewDataset(
    dataset_name="MyDataset",
    dataset_type=1,  # Computer Vision → dataset_tags_1
    dataset_short_info="Классификация изображений",
    dataset_tags=join_ml_tags("Image Classification", "my-project", "batch-a"),
    dataset_verification_type=1,
))

client.update_dataset(42, UpdateDatasetRequest(dataset_name="MyDataset-v2", remark="Переименован"))
client.delete_dataset(42, remark="Устарел")   # мягкое удаление
```

**Правила тегов:** каталог `GET …/system/config/dataset_tags_{dataset_type}` (тот же список при создании модели). Бэкенд требует ≥1 предопределённый ML-тег. Ставьте его **первым** — Label Studio / CVAT берут первый predefined. Custom-теги — после него. Хелперы: `join_ml_tags`, `list_predefined_ml_tags`.

---

## Метки

```python
client.add_dataset_labels(42, ["cat", "dog", "bird"])
labels = client.get_dataset_labels(42)
names = client.get_dataset_label_names(42)
client.update_dataset_label(42, label_id=7, label="feline")
```

---

## Сущности (загрузка образцов)

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
    file_paths=["/data/img001.jpg"],   # файлы, каталог или URL
)

client.update_dataset_entity(42, "entity-uuid", UpdateDatasetEntity(remark="Исправлена метка"))
entities = client.list_dataset_entities(42, version_id=7)
page = client.filter_dataset_entities(42, entity_name="sample", page=0, size=20)

client.delete_dataset_entities(["uuid-1", "uuid-2"], remark="Дубликаты удалены")
```

Передайте `file_category="input"|"output"` и опционально `split="train"|"validation"|"test"` при создании/обновлении. Файлы одной сущности — до **2 GB**.

---

## Массовая загрузка (Kappa ≥ 2.10.0)

Асинхронный job API. Пример: [`code_examples/bulk_upload.py`](../../code_examples/bulk_upload.py).

| Загрузка | Макс. размер | Примечание |
|---|---|---|
| CSV | **2 GB** (клиент) | На бэкенде по умолчанию **50 MB** (`BULK_UPLOAD_MAX_CSV_BYTES`) |
| Архив `.zip` | **50 GB** | `upload_type="archive"` + `archive_layout` + `dataset_schema` |

Для `input_output` нужен `dataset_schema={"inputDataPath": "input", ...}`; для `classes` — непустой список `classes`. Пустой `{}` даёт 422. Файл **стримится** с диска.

Поток операций с датасетом: [`dataset_operations_example.py`](../../code_examples/dataset_operations_example.py), lifecycle: [`dataset_lifecycle_example.py`](../../code_examples/dataset_lifecycle_example.py).

---

## Версии

```python
from kappa_apk import NewDatasetVersion

client.create_dataset_version(42, NewDatasetVersion(version_availability=1, version_remark="Первый релиз"))
versions = client.list_dataset_versions(42)
client.publish_dataset_version(42, "1.0.0", publish_type=2)
client.delete_dataset_version(42, "0.9.0")

details = client.get_dataset_version_details(dataset_name="MyDS", version_no="1.0.0")
```

---

## Загрузка и кэш

Архивы загружаются в `~/cache/kappa-framework/datasets/{dataset_name}_{version_no}/`. Повторные вызовы пропускают загрузку, если каталог кэша уже существует.

```python
info = client.download_dataset_version_archive(dataset_name="MyDS", version_no="1.0.0")
print(info.data_path)
```

При распаковке ZIP проверяются пути (защита от zip-slip через проверку префикса путей извлечения).

---

## Загрузка для обучения

```python
dataset = client.load_kappa_dataset(
    dataset_name="MyDS",
    version_no="1.0.0",
    transform=my_transform,
    transform_input_mode="content",   # или "path"
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
        x = batch[0]   # dict с entity_id и преобразованными полями
```

См. [LoadersAndTransforms.md](LoadersAndTransforms.md) для типов загрузчиков, dropout и преобразований.

[← Оглавление](README.md)
