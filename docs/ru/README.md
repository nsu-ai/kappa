# Документация SDK (`kf-sdk` / `kappa_apk`)

> **PyPI:** `kf-sdk` · **Импорт:** `kappa_apk` · **Версия:** 3.0.2 · **Ветка:** `main`  
> **Python:** 3.9+ · **API:** Kappa-framework **v2** (идентификация по JWT — без `user_id` / `user_type_id` в URL)
>
> Имя дистрибутива на PyPI **`kf-sdk`** сохранено для совместимости с ранними релизами; модуль импорта намеренно **`kappa_apk`**.

Python-клиент для ML-платформы [Kappa-framework](https://github.com/nsu-ai/kappa). Реализован на Rust + [PyO3](https://pyo3.rs); HTTP-запросы идут через **шлюз Traefik** по адресу `base_url`.

**Русская версия:** [README.md](../README.md) · **English:** [docs/README.md](../README.md)

---

## Оглавление

| Документ | Описание |
|---|---|
| [GettingStarted.md](GettingStarted.md) | Установка, настройка шлюза, первый скрипт |
| [KappaApkClient.md](KappaApkClient.md) | Справочник методов клиента + HTTP-пути v2 |
| [DataModels.md](DataModels.md) | Типы запросов/ответов (`Dataset`, `NewDataset`, …) |
| [Datasets.md](Datasets.md) | CRUD, метки, сущности, версии, загрузка и кэш |
| [LoadersAndTransforms.md](LoadersAndTransforms.md) | `KappaDataset`, `KappaDataLoader`, преобразования vision/text/audio |
| [Benchmarks.md](Benchmarks.md) | Рабочий процесс бенчмарков + `BenchmarkVerification` |
| [BuildAndPublish.md](BuildAndPublish.md) | Локальные wheels, cibuildwheel, PyPI / CI |

---

## Быстрый пример

```python
from kappa_apk import KappaApkClient

with KappaApkClient("https://kappa.nsu.ru:8061/", "user@example.com", "secret") as client:
    for batch in client.get_dataset_loader(dataset_name="MyDataset", version_no="1.0.0"):
        print(batch[0]["entity_id"])
```

---

## Структура пакета

```
kappa_apk/                  # нативное расширение (имя импорта)
├── KappaApkClient          # основной HTTP-клиент
├── KappaDataset            # датасет в памяти
├── KappaDataLoader         # пакетный загрузчик на Rust
├── Benchmarks              # через client.load_benchmark()
├── BenchmarkVerification   # проверка файлов бенчмарка
├── Dataset, DatasetItem, … # типизированные модели
├── Compose, Resize, …      # преобразования изображений (экспорт верхнего уровня)
├── BERTTokenizer, …        # текстовые преобразования
└── MelSpectrogram, …       # аудио-преобразования
```

Заглушки типов: [`kappa_apk.pyi`](../../kappa_apk.pyi) · Примеры: [`code_examples/`](../../code_examples/)
