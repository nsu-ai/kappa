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

# Только свои / назначенные / предоставленные, со статусом сборки версии
mine = client.filter_datasets(query_all=False, selected_version_no="1.0.0")
for item in mine["items"]:
    print(item["datasetName"], item.get("selectedVersionBuildStatus"))
```

Нумерация страниц начинается с 1, ответ имеет вид `{items, total, page, size, pages}`. `query_all` по умолчанию равен `True` и дополнительно включает публичный каталог. Передача `selected_version_id` или `selected_version_no` добавляет к каждому элементу `selectedVersionNo` и `selectedVersionBuildStatus` — дешевле, чем отдельный запрос версии для каждого датасета, если нужно лишь узнать, готова ли версия.

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
page = client.filter_dataset_entities(42, entity_name="sample", page=1, size=20)

# Только назначенные вам сущности, сначала новые
mine = client.filter_dataset_entities(
    42, assignment_filter="assigned", order_by="modifiedOn", order="DESC",
)

client.delete_dataset_entities(["uuid-1", "uuid-2"], remark="Дубликаты удалены")
```

Страницы сущностей нумеруются **с 1**, как и страницы датасетов, а направление сортировки задаётся через `order` (не `order_keyword`). Дополнительно доступны `entity_id`, `location_id`, `start_date` и `end_date`. Фильтра по split на сервере нет — читайте `dsEntityInfo.split` из каждого элемента.

Передайте `file_category="input"|"output"` и опционально `split="train"|"validation"|"test"` при создании/обновлении. Файлы одной сущности — до **2 GB**.

---

## Массовая загрузка (Kappa ≥ 2.11.0)

Асинхронный job API. Пример: [`code_examples/bulk_upload.py`](../../code_examples/bulk_upload.py).

| Загрузка | Макс. размер | Примечание |
|---|---|---|
| CSV | **2 GB** (клиент) | На бэкенде по умолчанию **50 MB** (`BULK_UPLOAD_MAX_CSV_BYTES`) |
| Архив `.zip` | **50 GB** | `upload_type="archive"` + `archive_layout` + `dataset_schema` |

Для `input_output` нужен `dataset_schema={"inputDataPath": "input", ...}`; для `classes` — непустой список `classes`. Пустой `{}` даёт 422. Файл **стримится** с диска.

Поток операций: [`dataset_operations_example.py`](../../code_examples/dataset_operations_example.py), lifecycle: [`dataset_lifecycle_example.py`](../../code_examples/dataset_lifecycle_example.py), мутации/сборки версий: [`bulk_mutation_and_version_build.py`](../../code_examples/bulk_mutation_and_version_build.py).

---

## Массовые мутации (Kappa ≥ 2.11.0)

Self-verify, auto-verify, mark-labeled, delete/recover/delete-files ставят асинхронный job (`202` + `jobId`). Опрос через `BulkMutationJob`. Одновременно — один активный mutation на датасет (**409**).

```python
start = client.mark_dataset_entities_labeled(42, all_eligible=True)
final = client.wait_for_bulk_mutation_job(42, start["jobId"])
client.bulk_self_verify_dataset_entities(42, status=1)  # 1=Pass, 3=Needs Modification
client.auto_verify_dataset_entities(42)
```

---

## Версии

На Kappa ≥ 2.11 create/refresh **ставят сборку архива**. Дождитесь готовности перед publish/download.

```python
from kappa_apk import NewDatasetVersion

created = client.create_dataset_version(
    42, NewDatasetVersion(version_availability=1, version_remark="Первый релиз")
)
build = client.wait_for_version_build_job(created["jobId"])
assert build.is_ready()
client.publish_dataset_version(42, created["versionNo"], publish_type=2)
versions = client.list_dataset_versions(42)
client.delete_dataset_version(42, "0.9.0")

details = client.get_dataset_version_details(dataset_name="MyDS", version_no="1.0.0")
```

---

## Загрузка и кэш

Используйте `download_dataset_version_package`: он читает манифест и качает шарды, а при отсутствии пакета (до 2.11 или один шард) откатывается на legacy-zip `download_dataset_version_archive`.

Архивы загружаются в `~/cache/kappa-framework/datasets/{dataset_name}_{version_no}/`. Повторные вызовы пропускают загрузку, если каталог кэша уже существует.

```python
info = client.download_dataset_version_package(dataset_name="MyDS", version_no="1.0.0")
print(info.data_path)
```

`load_kappa_dataset` и `get_dataset_loader` используют путь пакета автоматически. На Kappa ≥ 2.11 единый zip остаётся только для версий в один шард (по умолчанию 2 GB / 50 000 файлов), поэтому крупные версии доступны только через пакет.

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
