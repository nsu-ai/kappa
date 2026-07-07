# Loaders & transforms

**Русская версия:** [ru/LoadersAndTransforms.md](ru/LoadersAndTransforms.md)

---

## KappaDataset

PyTorch-style map-style dataset over downloaded samples.

```python
dataset = client.load_kappa_dataset(dataset_name="MyDS", version_no="1.0.0")
n = len(dataset)
sample = dataset[0]   # dict — includes entity_id
```

---

## KappaDataLoader

Rust iterable loader; **reshuffles each epoch** when `shuffle=True` (matches common PyTorch `DataLoader` epoch semantics).

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

### Entity dropout (bad-sample exclusion)

```python
loader.set_dropout_enabled(True)
loader.set_excluded_entity_ids(["bad-entity-1", "bad-entity-2"])
loader.clear_excluded_entity_ids()

loader.num_active_samples()   # samples after filter
loader.excluded_count()         # excluded count
```

If dropout would leave fewer than `min_remaining_entities` (default 1), the full dataset is kept and a Python `warnings.warn` is emitted.

---

## Framework adapters

| `loader_type` | Returns |
|---|---|
| `"kappa"` | `KappaDataLoader` (default) |
| `"pytorch"` | `torch.utils.data.DataLoader` |
| `"transformers"` | PyTorch `DataLoader` + `default_data_collator` |
| `"tensorflow"` | `tf.data.Dataset` (pass `tf_output_signature`) |

```python
helper_batch = DataLoaderHelper.peek_batch(loader)
sig = DataLoaderHelper.infer_tf_output_signature(sample_batch)
```

---

## Transforms

Registered as **top-level** `kappa_apk` exports (not subpackages).

### Vision

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

### Text

`SentencePieceTokenizer`, `VocabTransform`, `ToTensor`, `LabelToIndex`, `Truncate`, `AddToken`, `PadTransform`, `StrToIntTransform`, `GPT2BPETokenizer`, `CharBPETokenizer`, `CLIPTokenizer`, `BERTTokenizer`, `RegexTokenizer`, `Sequential`, `MaskTransform`

### Audio

`Spectrogram`, `GriffinLim`, `AmplitudeToDB`, `MelScale`, `InverseMelScale`, `MelSpectrogram`, `MFCC`, `MuLawEncoding`, `MuLawDecoding`, `Resample`, `ComplexNorm`, `TimeStretch`, `Fade`, `FrequencyMasking`, `TimeMasking`, `SlidingWindowCmn`, `Vad`

Pass a transform (or composed pipeline) to `load_kappa_dataset` / `get_dataset_loader`. Default `transform_input_mode="content"` passes file bytes/strings to transforms; use `"path"` to pass filesystem paths.

[← Index](README.md) · [Datasets](Datasets.md)
