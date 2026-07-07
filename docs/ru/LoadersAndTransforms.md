# Загрузчики и преобразования

**English:** [../LoadersAndTransforms.md](../LoadersAndTransforms.md)

---

## KappaDataset

Датасет в стиле PyTorch над загруженными образцами.

```python
dataset = client.load_kappa_dataset(dataset_name="MyDS", version_no="1.0.0")
n = len(dataset)
sample = dataset[0]   # dict — включает entity_id
```

---

## KappaDataLoader

Итерируемый загрузчик на Rust; **перемешивает данные каждую эпоху** при `shuffle=True` (соответствует типичной семантике эпох PyTorch `DataLoader`).

```python
loader = client.get_dataset_loader(
    dataset_name="MyDS",
    version_no="1.0.0",
    loader_type="kappa",
    batch_size=32,
    shuffle=True,
    drop_last=False,
)

for batch in loader:          # list[dict]
    ids = [s["entity_id"] for s in batch]
```

### Dropout сущностей (исключение плохих образцов)

```python
loader.set_dropout_enabled(True)
loader.set_excluded_entity_ids(["bad-entity-1", "bad-entity-2"])
loader.clear_excluded_entity_ids()

loader.num_active_samples()   # образцов после фильтра
loader.excluded_count()         # количество исключённых
```

Если после dropout останется меньше `min_remaining_entities` (по умолчанию 1), используется полный датасет и выводится `warnings.warn` в Python.

---

## Адаптеры фреймворков

| `loader_type` | Возвращает |
|---|---|
| `"kappa"` | `KappaDataLoader` (по умолчанию) |
| `"pytorch"` | `torch.utils.data.DataLoader` |
| `"transformers"` | PyTorch `DataLoader` + `default_data_collator` |
| `"tensorflow"` | `tf.data.Dataset` (передайте `tf_output_signature`) |

```python
helper_batch = DataLoaderHelper.peek_batch(loader)
sig = DataLoaderHelper.infer_tf_output_signature(sample_batch)
```

---

## Преобразования

Регистрируются как экспорты **верхнего уровня** `kappa_apk` (не подпакеты).

### Vision (изображения)

`Compose`, `LoadImage`, `Resize`, `CenterCrop`, `ToTensor`, `Normalize`, `RandomHorizontalFlip`, `RandomVerticalFlip`, `RandomCrop`

```python
from kappa_apk import Compose, Resize, CenterCrop, Normalize

transform = Compose([
    Resize((256, 256)),
    CenterCrop(224),
    Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225]),
])

loader = client.get_dataset_loader(..., transform=transform)
```

### Text (текст)

`SentencePieceTokenizer`, `VocabTransform`, `ToTensor`, `LabelToIndex`, `Truncate`, `AddToken`, `PadTransform`, `StrToIntTransform`, `GPT2BPETokenizer`, `CharBPETokenizer`, `CLIPTokenizer`, `BERTTokenizer`, `RegexTokenizer`, `Sequential`, `MaskTransform`

### Audio (аудио)

`Spectrogram`, `GriffinLim`, `AmplitudeToDB`, `MelScale`, `InverseMelScale`, `MelSpectrogram`, `MFCC`, `MuLawEncoding`, `MuLawDecoding`, `Resample`, `ComplexNorm`, `TimeStretch`, `Fade`, `FrequencyMasking`, `TimeMasking`, `SlidingWindowCmn`, `Vad`

Передайте преобразование (или составной конвейер) в `load_kappa_dataset` / `get_dataset_loader`. По умолчанию `transform_input_mode="content"` передаёт байты/строки файлов в преобразования; используйте `"path"` для путей файловой системы.

[← Оглавление](README.md) · [Датасеты](Datasets.md)
