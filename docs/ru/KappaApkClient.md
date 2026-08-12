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
| `filter_datasets(search?, dataset_tags?, dataset_type?, dataset_status?, publish_type?, page?, size?, order_by?, order_keyword?, query_all?, start_date?, end_date?, selected_version_id?, selected_version_no?)` | `GET …/datasets/filter` | Расширенные фильтры (по умолчанию: page=1, size=20) |
| `get_dataset_details(dataset_id?, dataset_name?)` | `GET …/datasets/filter` | Один `Dataset` |
| `get_dataset_fields(dataset_id)` | `GET …/datasets/fields/{id}` | Схема полей ввода |

`list_datasets` и `filter_datasets` обращаются к одному endpoint фильтрации с **разными значениями пагинации по умолчанию**. Нумерация страниц начинается с 1, ответ имеет вид `{items, total, page, size, pages}`.

- `query_all=False` ограничивает выборку датасетами, которыми вы владеете, на которые назначены или которые вам предоставлены. Значение по умолчанию (`True`) дополнительно включает публичный каталог.
- `selected_version_id` / `selected_version_no` добавляют к каждому элементу `selectedVersionNo` и `selectedVersionBuildStatus` — удобно проверить `ready` перед загрузкой.
- Направление сортировки здесь задаётся через `order_keyword`, а в фильтре сущностей — через `order`.

---

## Конфиг и ML-теги

| Метод | HTTP | Примечания |
|---|---|---|
| `get_system_config(tag)` | `GET /user-micro-services/v2/system/config/{tag}` | Например `dataset_type`, `dataset_tags_1` |
| `list_predefined_ml_tags(type_id)` | то же (`dataset_tags_{type_id}`) | Display-значения для форм создания |

Хелперы: `join_ml_tags(primary, *extras)`, `ensure_primary_ml_tag_first`, `validate_ml_tags`.

---

## CRUD датасетов

| Метод | HTTP |
|---|---|
| `add_dataset(dataset, check_tags=True)` | `POST …/datasets/new` |
| `update_dataset(dataset_id, update)` | `PUT …/datasets/{id}` |
| `delete_dataset(dataset_id, remark)` | `DELETE …/datasets/{id}` (мягкое удаление; восстановление через `/datasets/recover` на сервере) |

`check_tags=True` (по умолчанию) требует, чтобы **первый** тег был предопределённым для типа. То же для `create_model(..., check_tags=True)`.

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
| `filter_dataset_entities(dataset_id, entity_name?, entity_status?, version_id?, page?, size?, order_by?, order?, entity_id?, location_id?, assignment_filter?, start_date?, end_date?)` | `GET …/datasetEntities/filter/{id}` — страницы с 1; `assignment_filter` — `"assigned"` / `"not_assigned"` |
| `delete_dataset_entities(dataset_entity_ids, remark, version_id?)` | `DELETE …/datasetEntities` |

`file_paths` принимает пути к файлам, каталог (непосредственные дочерние элементы) или URL `http(s)://`.

---

## Версии и архивы

| Метод | HTTP |
|---|---|
| `get_dataset_version_details(dataset_id?, dataset_name?, version_id?, version_no?)` | `POST …/versions/details/list` или `GET …/versions/{id}/{ver}` |
| `download_dataset_version_archive(..., dataset_path?)` | `GET …/versions/archive/{id}/{ver}` |
| `download_dataset_version_package(...)` | `GET …/package` + shards |
| `create_dataset_version(dataset_id, version)` | `POST …/versions/new/{id}` → может вернуть `jobId` |
| `list_dataset_versions(dataset_id, version_availability?)` | `GET …/versions/{id}` |
| `delete_dataset_version(dataset_id, version_no)` | `DELETE …/versions/{id}/{ver}` |
| `publish_dataset_version(dataset_id, version_no, publish_type)` | `POST …/versions/publish/{id}/{ver}?publish_type=N` |
| `refresh_dataset_version` / `get_version_build_job` / `wait_for_version_build_job` | build-jobs API |

`publish_type`: `0` Not Published · `1` Private · `2` Open Source · `3` Public on Demand · `4` Purchase

На Kappa ≥ 2.11 дождитесь `buildStatus=ready` перед publish/download.

---

## Массовые мутации (Kappa ≥ 2.11)

| Метод | HTTP |
|---|---|
| `mark_dataset_entities_labeled(..., all_eligible?)` | `POST …/mark-labeled/{id}` |
| `bulk_self_verify_dataset_entities` / `auto_verify_dataset_entities` | verification endpoints |
| `wait_for_bulk_mutation_job` / `BulkMutationJob` | `…/bulk-mutation/jobs*` |

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
| `filter_benchmarks(...)` / `list_benchmarks()` | `GET /model-micro-services/v2/benchmarks` |
| `get_benchmark(id)` / `create_benchmark` / `update_benchmark` / `delete_benchmark` | `…/benchmarks[/{id}]` |
| `complete_benchmark_inference(id, model_version_id)` | `POST …/benchmarks/inferences/{id}/{version_id}` |
| `download_benchmark_dataset_package(id, dataset_id?, version_no?, dataset_path?)` | пакет датасета → прокси бенчмарка → legacy zip |
| `get_benchmark_dataset_package_manifest(id)` | `GET …/benchmarks/datasets/{id}/package` |
| `list_benchmark_remarks` / `add_benchmark_remark` | `…/benchmarks/{id}/remarks` |
| `get_benchmark_review` / `save_benchmark_review` / `finalize_benchmark_review` | `…/benchmarks/review/{expert_id}/{id}` |
| `get_benchmark_report` / `download_benchmark_report` / `regenerate_benchmark_report` | `…/benchmarks/report*` |
| `respond_to_benchmark_expert_request` / `assign_benchmark_expert` | `…/benchmarks/management*` |
| `propose_benchmark_dataset` / `confirm_benchmark_dataset` / `reject_benchmark_dataset` | `…/benchmarks/management/expert/{id}/dataset*` |
| `get_benchmark_dataset_attachments` / `get_benchmark_flow_schema` | `…/benchmarks/datasets/attachments/…`, `…/benchmarks/flow-schema` |

См. [Benchmarks.md](Benchmarks.md).

---

## Артефакты моделей

| Метод | HTTP |
|---|---|
| `write_model_inference(model_id, predictions?, metrics?, inference_result?, artifacts?, …)` | схема → валидация → `POST …/models/inferences/{model}` → загрузка артефактов |
| `upload_model_artifacts(model_id, inference_id, paths, …)` | по одной загрузке на файл, транспорт выбирается по размеру |
| `upload_model_inference_file(model_id, inference_id, file_path, file_category?, replace?, use_session?)` | `POST`/`PATCH …/models/inferences/files/{model}/{inference}` — для больших файлов переключается на сессию загрузки |
| `upload_model_artifact_session(..., on_progress?, checksum?, resume?, max_retries?, wait_for_slot?)` | `…/upload-session` → `…/part-urls` → presigned `PUT` → `…/complete` |
| `get_model_artifact_upload_session` / `abort_model_artifact_upload_session` | `…/upload-session/{upload_id}` |
| `get_model_inference_artifacts_package` / `get_model_version_artifacts_package` | `GET …/package` / `…/artifacts/package` |
| `download_model_inference_artifact_file(..., redirect?)` | `GET …/inferences/files/{model}/{inference}/{file_id}` |
| `download_model_inference_artifacts_package(..., dest_dir)` | манифест + загрузка по файлам |
| `download_model_inference_artifacts` / `download_model_version_artifacts` | `GET …/zip` — только небольшие пакеты |

`file_category`: `1` Training · `2` Inference · `3` Model · `4` Data · `5` Other. Обычная загрузка по умолчанию использует `2`, сессии загрузки и оба помощника выше — `3`.

Слишком большие обычные загрузки отклоняются с `413 USE_KAPPA_APK`, а пакеты сверх лимита zip отклоняют zip-маршрут с `409 PACKAGE_TOO_LARGE_FOR_ZIP`. SDK обрабатывает оба случая: большие загрузки автоматически переводятся на сессию, а `download_model_inference_artifacts_package` скачивает каждый файл отдельно. Для сессий на сервере должно быть включено объектное хранилище (иначе `503`), а число одновременных загрузок на модель ограничено (`429`) — параметр `wait_for_slot` пережидает лимит; отменяйте брошенные сессии, чтобы освободить слот быстрее.

### Запись инференса вместе с артефактами

`write_model_inference` делает всё за один вызов: читает эффективную схему инференса модели, формирует документ, валидирует его, создаёт инференс и загружает артефакты.

```python
written = client.write_model_inference(
    model_id,
    predictions=[
        {"entityId": item.entity_id, "original": item.annotations,
         "predicted": {"class_name": "pizza", "confidence": 0.98}}
    ],
    metrics={"accuracy": 0.93},
    artifacts=["./checkpoints", "./config.json"],   # файлы и/или каталоги
    on_progress=lambda name, sent, total, pct: print(f"{name} {pct}%"),
)
print(written["inferenceId"], written["schema"]["kind"])
```

В предсказаниях допускаются как `entity_id`, так и `entityId`, а строка в `predicted` оборачивается в ключ, которого требует схема (`class_name`, `text`, `answer`, …). Передайте `inference_result=…`, чтобы отправить собственный документ. Если схема отвергает данные, поднимается `ValueError` со списком путей JSON — до записи чего-либо на сервер.

### Большие веса (только через KappaApk)

В GUI загрузка артефактов ограничена: многогигабайтные веса модели можно загрузить только через этот SDK. Оба помощника это умеют:

```python
# Каталог шардов целиком в существующий инференс; повторный запуск дозальёт недостающее.
client.upload_model_artifacts(
    model_id, inference_id, "./llm-70b",     # model-00001-of-00042.safetensors, …
    on_progress=lambda name, sent, total, pct: print(f"{name} {pct}%"),
)

client.upload_model_inference_file(
    model_id, inference_id, "weights.safetensors", file_category=3,
)   # файл 8 ГБ → сессия загрузки, прозрачно

paths = client.download_model_inference_artifacts_package(
    model_id, inference_id, "./artifacts",
)
```

Сессии повторяют отправку каждой части по свежей presigned-ссылке и сохраняют ETag'и в кэш-каталоге пользователя, поэтому прерванная загрузка шарда на 100 ГБ продолжается с места остановки, а не с нуля (`resume=False` отключает). `upload_model_artifacts` загружает файлы по одному, укладываясь в лимит одновременных загрузок, и при `skip_existing` (по умолчанию) пропускает артефакты, уже приложенные с тем же именем и размером.

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
