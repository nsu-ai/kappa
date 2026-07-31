# Модели данных

**English:** [../DataModels.md](../DataModels.md)

Типизированные классы, экспортируемые из `kappa_apk`. Модели запросов реализуют `to_api_json()` для тел API в camelCase.

---

## Только для чтения (ответы API)

### `Dataset`

Метаданные датасета из списка / фильтра / поиска.

| Свойство | Тип | Примечания |
|---|---|---|
| `dataset_id` | `int` | |
| `dataset_name` | `str` | |
| `dataset_type` | `int` | 1 Vision · 2 Text · 3 Audio |
| `dataset_type_interp` | `str` | Человекочитаемый тип |
| `dataset_short_info` | `str` | |
| `dataset_status` | `int` | |
| `dataset_status_interp` | `str` | |
| `dataset_tags` | `str` | Через запятую. **Первый** тег — предопределённый ML-тег для `dataset_type` (`dataset_tags_{type}`); далее можно custom |
| `publish_type` | `int` | 0 Not Published · 1 Private · 2 Open Source · 3 Public on Demand · 4 Purchase |
| `created_on` / `modified_on` | `str` | |
| `version_no` | `str \| None` | Строка последней версии |

### `DatasetVersionDetails`

| Свойство | Тип |
|---|---|
| `id`, `user_id`, `dataset_id` | `int` |
| `version_availability` | `int` (1 = доступна) |
| `version_no`, `version_remark` | `str` |
| `publish_type` | `int` |
| `created_on`, `modified_on` | `str` |

### `DatasetDownloadDetails`

Результат загрузки и распаковки архива.

| Свойство | Тип |
|---|---|
| `dataset_id` | `int` |
| `version_no` | `str` |
| `data_path` | `str` | Локальный каталог кэша |
| `download_status` | `bool` | |

### `DatasetItem` / `ItemFile`

Один образец в версии датасета.

```python
item.entity_id          # str
item.files              # list[ItemFile] | None
item.annotations        # list[dict] | None

file.file_id            # str
file.file_name          # str
file.file               # str — абсолютный путь на диске
```

### `DatasetLabel`

| Свойство | Тип |
|---|---|
| `label_id`, `dataset_id` | `int` |
| `label` | `str` |
| `created_on`, `modified_on` | `str \| None` |

### `FileInformation` / `BenchmarkResult`

Используются при отправке бенчмарков. См. [Benchmarks.md](Benchmarks.md).

---

## Тела запросов (изменяемые)

| Класс | Назначение |
|---|---|
| `NewDataset` | `add_dataset` |
| `UpdateDatasetRequest` | `update_dataset` |
| `NewDatasetEntity` | `add_dataset_entity` |
| `UpdateDatasetEntity` | `update_dataset_entity` |
| `UpdateDatasetLabel` | `update_dataset_label` |
| `DeleteDatasetEntities` | `delete_dataset_entities` |
| `NewDatasetVersion` | `create_dataset_version` |

Все принимают эквивалентный обычный `dict` с ключами camelCase на границе клиента.

### Пример: `NewDataset`

```python
from kappa_apk import NewDataset, join_ml_tags

ds = NewDataset(
    dataset_name="MyDataset",
    dataset_type=1,
    dataset_short_info="Классификация изображений",
    dataset_tags=join_ml_tags("Image Classification", "cls"),
    dataset_verification_type=1,
)
client.add_dataset(ds)
```

### Пример: `NewDatasetEntity`

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

## Объекты времени выполнения

| Класс | Роль |
|---|---|
| `KappaDataset` | Датасет в памяти (`__len__`, `__getitem__`) |
| `KappaDataLoader` | Итератор пакетов с перемешиванием по эпохам |
| `Benchmarks` | Возвращается `load_benchmark()` — не импортируется напрямую |
| `BenchmarkVerification` | Автономная проверка файлов |
| `DataLoaderHelper` | Статические утилиты (`peek_batch`, вывод сигнатуры TF) |

[← Оглавление](README.md) · [Руководство по датасетам](Datasets.md)
