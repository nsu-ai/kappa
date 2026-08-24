# Бенчмарки

**English:** [../Benchmarks.md](../Benchmarks.md)

---

## Обзор рабочего процесса

```python
bm = client.load_benchmark("eaa50325-5f3d-4e66-b7b5-b18e5a587563")

data = bm.dataset()              # загрузка и кэш датасета бенчмарка
bm.setup_project()               # опционально: отпечатки файлов src/

predictions = [
    {"entity_id": item.entity_id, "predicted": {"label": ...}}
    for item in data
]
metrics = {"accuracy": 0.95, "f1": 0.92}

result = bm.save_benchmark(predictions, metrics, model_path="./model")
response = bm.submit_benchmark(upload_artifacts=True)   # инференс + веса + привязка
```

`Benchmarks` возвращается `load_benchmark()` — **не** экспортируется как `kappa_apk.Benchmarks`.

---

## Методы Benchmarks

| Метод | Описание |
|---|---|
| `benchmark_id` | Свойство UUID |
| `dataset(dataset_path?)` | Загрузка оценочного набора → `list[DatasetItem]`; кэш в каталоге кэша ОС (`kappa-framework/benchmarks/{id}/`) |
| `setup_project()` | Отпечатки файлов из ближайшего каталога `src/` |
| `set_model_id(model_id)` / `get_model_id()` | Переопределение ID модели для отправки |
| `debug_benchmark_details()` | Человекочитаемая строка статуса |
| `save_benchmark(predictions, metrics?, model_path?)` | Формирование `BenchmarkResult` в памяти |
| `saved_result` | Последний сохранённый `BenchmarkResult` или `None` |
| `submit_benchmark(strict?, model_version_id?, complete_inference?, create_version?, upload_artifacts?, artifact_paths?, on_progress?, attach_pipeline?, pipeline?, pipeline_type?, model?, entrypoint?)` | `POST …/models/inferences/{model_id}`, artifact upload, **pipeline draft PUT** (Kappa ≥ 2.14, before version create), then link |

`save_benchmark` принимает результаты только с predictions, или прикрепляет метаданные модели/приложения из `setup_project()`, `model_path` или ранее установленного `set_model_path`. Обязательные ключи `predicted` берутся из схемы инференса модели (на Kappa ≥ 2.14 — выходы датасета, например `label` / `output_text`). Лишние ключи вроде `confidence` сохраняются. `original` не обязателен и не используется как эталон. Метрики — словарь имя → число; Kappa ≥ 2.14 принимает незарегистрированные числовые имена, а старые бэкенды могут ответить 400, если ключа нет в схеме.

После загрузки артефактов `submit_benchmark` определяет pipeline по запущенной программе (`sys.modules`, `__main__.__file__`, опционально `model=`) и файлам весов и делает PUT на инференс **до** создания версии, чтобы Kappa ≥ 2.14 мог его продвинуть. На шлюзах до 2.14 маршрут отвечает 404 — отправка всё равно проходит. `attach_pipeline=False` отключает детект; `pipeline={...}` задаёт тело вручную (`source: manual`) — туда можно положить архитектуру, классы и препроцесс. Автодетект пишет только framework / имя файла весов / скрипт и **не** запускает инференс. Смесь `.pt`+`.onnx` без явного тела не угадывается.

Сохранение инференса само по себе оставляет бенчмарк в статусе *Pending Inference*. Поэтому `submit_benchmark()` дополнительно вызывает `POST …/benchmarks/inferences/{benchmark_id}/{model_version_id}`, что переводит бенчмарк в *Inference Completed*. Этой привязке нужна **версия модели**, иначе сервер отвечает `404 MODEL_VERSION_NOT_FOUND`. Версия определяется по порядку:

1. `model_version_id`, если передан;
2. `mlmodelVersionId` бенчмарка, если версию уже назначил мейнтейнер (у новых бенчмарков там `0` — не подходит);
3. версия, содержащая только что отправленный инференс: при `create_version=True` (по умолчанию) она создаётся через `POST …/models/versions/{model_id}`, если её ещё нет.

Передайте `create_version=False`, чтобы вместо неявного создания версии получить ошибку, или `complete_inference=False`, чтобы отправить результаты без привязки.

`save_benchmark(model_path=…)` записывает только *метаданные* файлов (имя, размер, хэш). Чтобы сохранить сами веса, добавьте `upload_artifacts=True`:

```python
bm.save_benchmark(predictions, metrics, model_path="./model")
bm.submit_benchmark(
    upload_artifacts=True,                     # загрузит все файлы из ./model
    on_progress=lambda name, sent, total, pct: print(f"{name} {pct}%"),
)
```

Артефакты по умолчанию получают `file_category=3` (Model); файлы сверх серверного лимита обычной загрузки идут через возобновляемую multipart-сессию, поэтому чекпойнты в несколько ГБ работают. Передайте `artifact_paths=[…]`, чтобы загрузить что-то помимо сохранённого `model_path`. Для обычного (не бенчмарк) запуска то же самое делает `client.write_model_inference()` за один вызов — см. [KappaApkClient.md](KappaApkClient.md#артефакты-моделей).

---

## Загрузка оценочного набора (Kappa ≥ 2.11.0)

`bm.dataset()` и `client.download_benchmark_dataset_package()` повторяют порядок веб-интерфейса:

1. **Пакет версии датасета** — `GET /data-micro-services/v2/datasets/versions/{datasetId}/{versionNo}/package` с `datasetId` / `datasetVersionNo` из деталей бенчмарка. Многошардовые манифесты загружаются шард за шардом.
2. **Прокси бенчмарка** — при `401` / `403` / `404` (есть `benchmark.read`, но нет `dataset.read`) повтор через `GET /model-micro-services/v2/benchmarks/datasets/{benchmark_id}/package` и `…/package/shards/{name}`.
3. **Legacy zip** — любой путь использует единый архив, если манифест сообщает `legacySingleZip` или содержит не более одного шарда.

Для больших версий 2.11 единого zip уже нет, поэтому прямой вызов legacy-маршрута загрузки бенчмарка вернёт `404`.

| Ситуация | Результат |
|---|---|
| Архив версии ещё собирается | `RuntimeError` — дождитесь задания сборки версии и повторите |
| Запрос на загрузку ожидает подтверждения (`publish_type = 3`) | `PermissionError` |
| Нет и `benchmark.read` | Ошибка от прокси-запроса |

```python
path = client.download_benchmark_dataset_package("eaa50325-…")   # каталог кэша
manifest = client.get_benchmark_dataset_package_manifest("eaa50325-…")
```

---

## Управление бенчмарками

| Метод | Endpoint |
|---|---|
| `filter_benchmarks(...)` / `list_benchmarks()` | `GET /benchmarks` — `benchmark_id`, `model_id`, `dataset_id`, `benchmark_status`, `user_id`, диапазон дат, `order_by` / `order`, `page` / `size` |
| `get_benchmark(id)` | `GET /benchmarks/{id}` — включает `datasetVersionNo` и `benchmarkStatus` |
| `create_benchmark(body)` / `update_benchmark(id, body)` / `delete_benchmark(id)` | `POST` / `PUT` / `DELETE /benchmarks[/{id}]` |
| `complete_benchmark_inference(id, model_version_id)` | `POST /benchmarks/inferences/{id}/{model_version_id}` |
| `list_benchmark_remarks(id)` / `add_benchmark_remark(id, msg)` | `GET` / `POST /benchmarks/{id}/remarks` (1–4000 символов) |
| `get_benchmark_flow_schema()` | `GET /benchmarks/flow-schema` |

### Эксперт и выбор датасета

| Метод | Endpoint |
|---|---|
| `respond_to_benchmark_expert_request(id, accept)` | `POST /benchmarks/management/expert/{id}/{2 или 0}` |
| `assign_benchmark_expert(id, expert_id)` | `POST /benchmarks/management/{id}/{expert_id}` — нужен `benchmark.manage` |
| `propose_benchmark_dataset(id, dataset_id, version_id)` | `POST /benchmarks/management/expert/{id}/{dataset_id}/{version_id}` |
| `confirm_benchmark_dataset(id)` / `reject_benchmark_dataset(id)` | `POST …/dataset/confirm` / `…/dataset/reject-proposal` |
| `get_benchmark_dataset_attachments(dataset_id, version_id)` | `GET /benchmarks/datasets/attachments/{dataset_id}/{version_id}` |

### Экспертиза и отчёт

| Метод | Endpoint |
|---|---|
| `get_benchmark_review(id, expert_id)` | `GET /benchmarks/review/{expert_id}/{id}` — первый вызов переводит статус 5 → 6 |
| `save_benchmark_review(id, expert_id, review)` | `PUT` тот же путь |
| `finalize_benchmark_review(id, expert_id)` | `POST` тот же путь — планирует генерацию отчёта |
| `regenerate_benchmark_report(id)` | `PATCH /benchmarks/review/report/{id}` — нужен `benchmark.manage` |
| `get_benchmark_report(id)` | `GET /benchmarks/report/{id}` |
| `download_benchmark_report(id, dest_path, lang?)` | `GET /benchmarks/report/download/{id}/{lang}` — `"en"` (по умолчанию) или `"ru"` |

```python
review = client.get_benchmark_review(benchmark_id, expert_id)
client.save_benchmark_review(benchmark_id, expert_id, {
    "reviews": {"<entity-uuid>": {"entityId": "<entity-uuid>", "score": 4, "remark": "ok"}},
    "finalScore": 4,
})
client.finalize_benchmark_review(benchmark_id, expert_id)
client.download_benchmark_report(benchmark_id, "report.pdf", lang="ru")
```

---

## BenchmarkVerification

Автономный помощник для проверки файлов приложения по требованиям сервера (публичные endpoints проверки — JWT не требуется).

```python
from kappa_apk import BenchmarkVerification

bv = BenchmarkVerification(
    server_url="https://kappa.nsu.ru:8061/",
    benchmark_id="8c97da09-375c-47cd-814c-d1798dbd48f4",
    path="/path/to/project",
)
ok = bv.result()   # bool — сканирует каталог, отправляет хэши в API проверки v2
```

HTTP (через шлюз):

- `GET …/model-micro-services/v2/benchmarks/app/verifications/files/{benchmark_id}`
- `POST …/model-micro-services/v2/benchmarks/app/verifications/{benchmark_id}`

Отсутствие `valid` / `success` в JSON-ответе трактуется как **неудача** (`False`).

---

## Кэширование

| Ресурс | Путь кэша |
|---|---|
| Оценочный набор бенчмарка | Кэш ОС `kappa-framework/benchmarks/{benchmark_id}/` (старый `~/cache/…` переиспользуется, если уже полный) |
| Файлы запуска модели | Из `model_path` или метаданных `setup_project()` |

Загрузка помечается кэшированной только после распаковки всех шардов, поэтому прерванная загрузка повторяется с начала, а не остаётся неполной.

[← Оглавление](README.md)
