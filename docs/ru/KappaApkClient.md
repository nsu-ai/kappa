# KappaApkClient

**English:** [../KappaApkClient.md](../KappaApkClient.md)

Основная точка входа в SDK. Обёртка над аутентифицированным HTTP к микросервисам Kappa-framework **v2** через шлюз Traefik.

```python
from kappa_apk import KappaApkClient

client = KappaApkClient(base_url: str, login_id: str, passwd: str)
```

Пути ниже добавляются к `base_url`. Все аутентифицированные вызовы датасетов и бенчмарков отправляют `Authorization: Bearer <token>`.

---

## Сессия и клиент

| Метод | HTTP | Описание |
|---|---|---|
| `connect()` | `POST /user-micro-services/v2/session/new` | Вход; возвращает словарь с данными пользователя и токеном |
| `close()` | `DELETE /user-micro-services/v2/session` | Выход; инвалидация токена (только Bearer) |
| `get_token()` | — | Текущий bearer-токен или `None` |
| `is_authenticated()` | — | Есть ли сохранённый токен |
| `get_base_url()` / `set_base_url(url)` | — | Получение/установка URL шлюза |
| `__enter__` / `__exit__` | — | Контекстный менеджер: вход при входе, выход при выходе |
| `make_request(method, endpoint, data?, token?)` | *произвольный* | Низкоуровневый аутентифицированный запрос |

---

## Список и поиск датасетов

| Метод | HTTP | Примечания |
|---|---|---|
| `list_datasets(page?, size?, order_by?, order_keyword?)` | `GET /data-micro-services/v2/datasets/filter` | Страница в виде JSON (по умолчанию: page=1, size=200) |
| `list_datasets_typed(...)` | то же | Возвращает `list[Dataset]` |
| `filter_datasets(search?, dataset_tags?, dataset_type?, dataset_status?, publish_type?, page?, size?, order_by?, order?)` | `GET …/datasets/filter` | Расширенные фильтры (по умолчанию: page=1, size=20) |
| `get_dataset_details(dataset_id?, dataset_name?)` | `GET …/datasets/filter` | Один `Dataset` |
| `get_dataset_fields(dataset_id)` | `GET …/datasets/fields/{id}` | Схема полей ввода |

`list_datasets` и `filter_datasets` обращаются к одному endpoint фильтрации с **разными значениями пагинации по умолчанию**.

---

## CRUD датасетов

| Метод | HTTP |
|---|---|
| `add_dataset(dataset)` | `POST …/datasets/new` |
| `update_dataset(dataset_id, update)` | `PUT …/datasets/{id}` |
| `delete_dataset(dataset_id, remark)` | `DELETE …/datasets/{id}` (мягкое удаление; восстановление через `/datasets/recover` на сервере) |

`dataset` / `update` принимают типизированные модели или обычный `dict` (ключи JSON в camelCase).

---

## Метки

| Метод | HTTP |
|---|---|
| `add_dataset_labels(dataset_id, labels)` | `POST …/datasets/labels/{id}` |
| `get_dataset_labels(dataset_id)` | `GET …/datasets/labels/{id}` → `list[DatasetLabel]` |
| `get_dataset_label_names(dataset_id)` | `GET …/datasets/labels/{id}` → `list[str]` |
| `update_dataset_label(dataset_id, label_id, label)` | `PUT …/datasets/labels/{id}` |

---

## Сущности (образцы)

| Метод | HTTP |
|---|---|
| `add_dataset_entity(dataset_id, entity, file_paths?, version_id?)` | `POST …/datasetEntities/new/{id}` |
| `update_dataset_entity(dataset_id, entity_id, update, version_id?)` | `PUT …/datasetEntities/{id}/{entity_id}` |
| `list_dataset_entities(dataset_id, version_id?)` | `GET …/datasetEntities/{id}` |
| `get_dataset_entity(dataset_id, entity_id, version_id?)` | `GET …/datasetEntities/{id}/{entity_id}` |
| `filter_dataset_entities(dataset_id, entity_name?, entity_status?, version_id?, page?, size?, order_by?, order?)` | `GET …/datasetEntities/filter/{id}` |
| `delete_dataset_entities(dataset_entity_ids, remark, version_id?)` | `DELETE …/datasetEntities` |

`file_paths` принимает пути к файлам, каталог (непосредственные дочерние элементы) или URL `http(s)://`.

---

## Версии и архивы

| Метод | HTTP |
|---|---|
| `get_dataset_version_details(dataset_id?, dataset_name?, version_id?, version_no?)` | `POST …/versions/details/list` или `GET …/versions/{id}/{ver}` |
| `download_dataset_version_archive(..., dataset_path?)` | `GET …/versions/archive/{id}/{ver}` |
| `create_dataset_version(dataset_id, version)` | `POST …/versions/new/{id}` |
| `list_dataset_versions(dataset_id, version_availability?)` | `GET …/versions/{id}` |
| `delete_dataset_version(dataset_id, version_no)` | `DELETE …/versions/{id}/{ver}` |
| `publish_dataset_version(dataset_id, version_no, publish_type)` | `POST …/versions/publish/{id}/{ver}?publish_type=N` |

`publish_type`: `0` Private · `1` Internal · `2` Public

---

## Вспомогательные методы для обучения

| Метод | Описание |
|---|---|
| `load_kappa_dataset(..., transform?, transform_input_mode?)` | Загрузка (с кэшем) + `KappaDataset` |
| `get_dataset_loader(..., loader_type?, batch_size?, shuffle?, drop_last?, transform?, tf_output_signature?)` | Загрузчик для конкретного фреймворка |

`loader_type`: `"kappa"` (по умолчанию) · `"pytorch"` · `"transformers"` · `"tensorflow"`

Корень кэша: `~/cache/kappa-framework/datasets/{name}_{version}/`

---

## Бенчмарки

| Метод | HTTP |
|---|---|
| `load_benchmark(benchmark_id)` | Возвращает объект `Benchmarks` (загрузка через API бенчмарков v2) |

См. [Benchmarks.md](Benchmarks.md).

---

## Пример

```python
with KappaApkClient(gateway, user, pwd) as client:
    page = client.list_datasets(size=50)
    ds = client.get_dataset_details(dataset_name="MNIST")
    loader = client.get_dataset_loader(dataset_name="MNIST", version_no="1.0.0", batch_size=64)
    for batch in loader:
        process(batch)
```

[← Оглавление](README.md) · [Модели данных](DataModels.md)
