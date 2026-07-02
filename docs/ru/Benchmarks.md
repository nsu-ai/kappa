# Бенчмарки

**English:** [../Benchmarks.md](../Benchmarks.md)

---

## Обзор рабочего процесса

```python
bm = client.load_benchmark("eaa50325-5f3d-4e66-b7b5-b18e5a587563")

data = bm.dataset()              # загрузка и кэш датасета бенчмарка
bm.setup_project()               # опционально: отпечатки файлов src/

predictions = [
    {"entity_id": item.entity_id, "original": ..., "predicted": ...}
    for item in data
]
metrics = {"accuracy": 0.95, "f1": 0.92}

result = bm.save_benchmark(predictions, metrics)
response = bm.submit_benchmark()   # POST результата инференса в model service
```

`Benchmarks` возвращается `load_benchmark()` — **не** экспортируется как `kappa_apk.Benchmarks`.

---

## Методы Benchmarks

| Метод | Описание |
|---|---|
| `benchmark_id` | Свойство UUID |
| `dataset(dataset_path?)` | Загрузка архива → `list[DatasetItem]`; кэш в `~/cache/kappa-framework/benchmarks/{id}/` |
| `setup_project()` | Отпечатки файлов из ближайшего каталога `src/` |
| `set_model_id(model_id)` / `get_model_id()` | Переопределение ID модели для отправки |
| `debug_benchmark_details()` | Человекочитаемая строка статуса |
| `save_benchmark(predictions, metrics?, model_path?)` | Формирование `BenchmarkResult` в памяти |
| `submit_benchmark()` | `POST /model-micro-services/v2/models/inferences/{model_id}` |

`save_benchmark` принимает результаты только с predictions, или прикрепляет метаданные модели/приложения из `setup_project()`, `model_path` или ранее установленного `set_model_path`.

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
| Датасет бенчмарка | `~/cache/kappa-framework/benchmarks/{benchmark_id}/` |
| Файлы запуска модели | Из `model_path` или метаданных `setup_project()` |

[← Оглавление](README.md)
