#!/usr/bin/env python3
# Copyright 2026 Kappa-Apk @nsu.ru
# SPDX-License-Identifier: Apache-2.0
"""
Train a small CNN on Fashion-MNIST loaded from Kappa via ``KappaDataLoader``.

Uses ``KappaApkClient.get_dataset_loader(..., loader_type="kappa")`` with grayscale
28×28 transforms. Each batch includes ``entity_id`` per sample for downstream tooling.

Prerequisite: upload Fashion-MNIST with ``upload_torchvision_to_kappa_example.py``::

    python upload_torchvision_to_kappa_example.py --dataset fashion_mnist \\
        --base-url http://127.0.0.1:8060 --login-id admin --password '***' \\
        --limit-per-class 50

Run::

    source code_examples/.venv/bin/activate
    python mnist_kappa_training_example.py \\
        --base-url http://127.0.0.1:8060 --login-id admin --password '***'

    python mnist_kappa_training_example.py --epochs 3 --max-batches 20
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Any

try:
    import torch
    import torch.nn as nn
    from torchvision import transforms
    from torchvision.datasets import FashionMNIST
except ImportError as e:
    print("Missing PyTorch/torchvision. Run ./code_examples/setup_venv.sh", file=sys.stderr)
    raise SystemExit(1) from e

try:
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
except ImportError as e:
    print("Missing matplotlib. Run ./code_examples/setup_venv.sh", file=sys.stderr)
    raise SystemExit(1) from e

try:
    from kappa_apk import KappaApkClient
except ImportError as e:
    print(
        "Missing kappa_apk. Build and install:\n"
        "  ./code_examples/setup_venv.sh\n"
        "  source code_examples/.venv/bin/activate",
        file=sys.stderr,
    )
    raise SystemExit(1) from e

OUTPUT_DIR = Path(__file__).resolve().parent / "output"
NUM_CLASSES = 10
DEVICE = torch.device("cuda" if torch.cuda.is_available() else "cpu")
SEED = 42

_CLASS_NAMES: tuple[str, ...] | None = None
_CLASS_TO_IDX: dict[str, int] | None = None
_NORM_TO_IDX: dict[str, int] | None = None


class FashionMNISTCNN(nn.Module):
    """Small CNN for 1×28×28 inputs (~200k parameters)."""

    def __init__(self, num_classes: int = NUM_CLASSES) -> None:
        super().__init__()
        self.features = nn.Sequential(
            nn.Conv2d(1, 32, kernel_size=3, padding=1),
            nn.ReLU(inplace=True),
            nn.MaxPool2d(2),
            nn.Conv2d(32, 64, kernel_size=3, padding=1),
            nn.ReLU(inplace=True),
            nn.MaxPool2d(2),
            nn.Conv2d(64, 64, kernel_size=3, padding=1),
            nn.ReLU(inplace=True),
        )
        self.classifier = nn.Sequential(
            nn.Flatten(),
            nn.Linear(64 * 7 * 7, 128),
            nn.ReLU(inplace=True),
            nn.Dropout(0.25),
            nn.Linear(128, num_classes),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.classifier(self.features(x))


def label_display(class_name: str) -> str:
    return class_name.replace("_", " ").strip()


def normalize_label_key(name: str) -> str:
    s = str(name).strip().lower().replace("/", " ").replace("_", " ")
    return re.sub(r"\s+", " ", s).strip()


def label_variants(name: str) -> list[str]:
    variants = [
        name,
        name.lower(),
        label_display(name),
        label_display(name).lower(),
        name.replace("/", " "),
        name.replace(" ", "/"),
    ]
    seen: set[str] = set()
    out: list[str] = []
    for v in variants:
        if v and v not in seen:
            seen.add(v)
            out.append(v)
    return out


def init_label_index(client: KappaApkClient, dataset_id: int | None) -> int:
    global _CLASS_NAMES, _CLASS_TO_IDX, _NORM_TO_IDX, NUM_CLASSES

    kappa_labels = client.get_dataset_label_names(dataset_id) if dataset_id else []
    if kappa_labels:
        canonical = tuple(kappa_labels)
        print(f"Using {len(canonical)} Kappa dataset labels for class index")
    else:
        canonical = tuple(FashionMNIST.classes)
        print("Kappa labels unavailable; using torchvision FashionMNIST.classes")

    mapping: dict[str, int] = {}
    norm_map: dict[str, int] = {}

    def register(name: str, index: int) -> None:
        for variant in label_variants(name):
            mapping.setdefault(variant, index)
            norm_map.setdefault(normalize_label_key(variant), index)

    for i, name in enumerate(canonical):
        register(name, i)

    for tv_name in FashionMNIST.classes:
        tv_norm = normalize_label_key(tv_name)
        for i, kappa_name in enumerate(canonical):
            if normalize_label_key(kappa_name) == tv_norm:
                register(tv_name, i)
                break

    _CLASS_NAMES = canonical
    _CLASS_TO_IDX = mapping
    _NORM_TO_IDX = norm_map
    NUM_CLASSES = len(canonical)
    return NUM_CLASSES


def resolve_label_index(key: str) -> int | None:
    assert _CLASS_TO_IDX is not None and _NORM_TO_IDX is not None
    if key in _CLASS_TO_IDX:
        return _CLASS_TO_IDX[key]
    low = key.lower()
    if low in _CLASS_TO_IDX:
        return _CLASS_TO_IDX[low]
    return _NORM_TO_IDX.get(normalize_label_key(key))


def fashion_mnist_transform() -> transforms.Compose:
    return transforms.Compose(
        [
            transforms.Grayscale(num_output_channels=1),
            transforms.Resize((28, 28)),
            transforms.ToTensor(),
            transforms.Normalize((0.5,), (0.5,)),
        ]
    )


def sample_split(raw: dict[str, Any]) -> str | None:
    info = raw.get("ds_entity_info") or raw.get("dsEntityInfo")
    if isinstance(info, dict):
        split = info.get("split")
        if split is not None:
            return str(split).strip().lower()
    return None


def label_from_raw(raw: dict[str, Any]) -> int:
    annotations = raw.get("annotations") or []
    if isinstance(annotations, list):
        for ann in annotations:
            if not isinstance(ann, dict) or ann.get("type") != "choices":
                continue
            choices = (ann.get("value") or {}).get("choices") or []
            if choices:
                idx = resolve_label_index(str(choices[0]).strip())
                if idx is not None:
                    return idx

    info = raw.get("ds_entity_info") or raw.get("dsEntityInfo")
    if isinstance(info, dict) and info.get("label") is not None:
        idx = resolve_label_index(str(info["label"]).strip())
        if idx is not None:
            return idx

    files = raw.get("files") or []
    if isinstance(files, list):
        for item in files:
            path_str = ""
            if isinstance(item, dict):
                path_str = str(item.get("file") or item.get("file_name") or "")
            else:
                path_str = str(getattr(item, "file", None) or getattr(item, "file_name", "") or "")
            for part in reversed(Path(path_str.replace("\\", "/")).parts):
                idx = resolve_label_index(part)
                if idx is not None:
                    return idx

    raise ValueError(f"Could not resolve label for entity_id={raw.get('entity_id')!r}")


def make_kappa_loader(
    client: KappaApkClient,
    dataset_name: str,
    version_no: str | None,
    batch_size: int,
) -> Any:
    def target_transform(raw: dict[str, Any]) -> int:
        return label_from_raw(raw)

    return client.get_dataset_loader(
        dataset_name=dataset_name,
        version_no=version_no,
        loader_type="kappa",
        batch_size=batch_size,
        shuffle=True,
        drop_last=False,
        transform=fashion_mnist_transform(),
        target_transform=target_transform,
        transform_input_mode="content",
    )


def prepare_batch(
    batch: list[dict[str, Any]],
    train_only: bool,
) -> tuple[torch.Tensor, torch.Tensor] | None:
    xs: list[torch.Tensor] = []
    ys: list[int] = []
    for sample in batch:
        if train_only:
            split = sample_split(sample)
            if split is not None and split != "train":
                continue
        x = sample["x"]
        if not isinstance(x, torch.Tensor):
            raise TypeError(f"Expected tensor x after transform, got {type(x)}")
        if x.dim() == 3 and x.size(0) == 3:
            x = x.mean(dim=0, keepdim=True)
        y = sample.get("y")
        if y is None:
            raise KeyError("Missing 'y' — ensure target_transform is set on the loader")
        xs.append(x)
        ys.append(int(y))
    if not xs:
        return None
    return torch.stack(xs).to(DEVICE), torch.tensor(ys, dtype=torch.long, device=DEVICE)


def train_epoch(
    model: nn.Module,
    loader: Any,
    optimizer: torch.optim.Optimizer,
    criterion: nn.CrossEntropyLoss,
    train_only: bool,
    max_batches: int | None,
) -> tuple[float, float]:
    model.train()
    running_loss = 0.0
    n_batches = 0
    correct = 0
    total = 0

    for batch_idx, batch in enumerate(loader):
        if max_batches is not None and batch_idx >= max_batches:
            break
        prepared = prepare_batch(batch, train_only=train_only)
        if prepared is None:
            continue
        images, labels = prepared
        optimizer.zero_grad(set_to_none=True)
        logits = model(images)
        loss = criterion(logits, labels)
        loss.backward()
        optimizer.step()

        preds = logits.argmax(dim=1)
        correct += int((preds == labels).sum().item())
        total += labels.size(0)
        running_loss += float(loss.item())
        n_batches += 1

    avg_loss = running_loss / max(n_batches, 1)
    accuracy = correct / total if total > 0 else 0.0
    return avg_loss, accuracy


def save_curves(
    epoch_losses: list[float],
    epoch_accuracies: list[float],
    dataset_name: str,
) -> tuple[Path, Path]:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    epochs = list(range(len(epoch_losses)))
    acc_pct = [a * 100.0 for a in epoch_accuracies]

    fig, (ax_loss, ax_acc) = plt.subplots(1, 2, figsize=(10, 4))
    fig.suptitle(f"Fashion-MNIST Kappa training ({dataset_name})", fontsize=12)

    ax_loss.plot(epochs, epoch_losses, marker="o", color="#2563eb", linewidth=2)
    ax_loss.set_xlabel("Epoch")
    ax_loss.set_ylabel("Training loss")
    ax_loss.grid(True, alpha=0.3)

    ax_acc.plot(epochs, acc_pct, marker="o", color="#16a34a", linewidth=2)
    ax_acc.set_xlabel("Epoch")
    ax_acc.set_ylabel("Training accuracy (%)")
    ax_acc.set_ylim(0, 100)
    ax_acc.grid(True, alpha=0.3)

    fig.tight_layout()
    plot_path = OUTPUT_DIR / "mnist_kappa_training_curves.png"
    fig.savefig(plot_path, dpi=150, bbox_inches="tight")
    plt.close(fig)

    metrics_path = OUTPUT_DIR / "mnist_kappa_training_metrics.json"
    metrics_path.write_text(
        json.dumps(
            {
                "dataset_name": dataset_name,
                "epochs": [
                    {"epoch": i, "loss": epoch_losses[i], "accuracy_percent": round(acc_pct[i], 4)}
                    for i in epochs
                ],
                "plot_path": str(plot_path),
            },
            indent=2,
        ),
        encoding="utf-8",
    )
    return plot_path, metrics_path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Train a small CNN on Fashion-MNIST via KappaDataLoader.",
    )
    parser.add_argument(
        "--base-url",
        default=os.environ.get("KAPPA_BASE_URL", "http://127.0.0.1:8060"),
    )
    parser.add_argument("--login-id", default=os.environ.get("KAPPA_LOGIN_ID", "admin"))
    parser.add_argument("--password", default=os.environ.get("KAPPA_PASSWORD", ""))
    parser.add_argument("--dataset-name", default="FashionMNIST")
    parser.add_argument("--version-no", default=None)
    parser.add_argument("--epochs", type=int, default=5)
    parser.add_argument("--batch-size", type=int, default=128)
    parser.add_argument("--lr", type=float, default=0.001)
    parser.add_argument(
        "--train-only",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Use only entities tagged with split=train (default: true)",
    )
    parser.add_argument(
        "--max-batches",
        type=int,
        default=None,
        help="Cap batches per epoch for smoke tests",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if not args.password:
        print("Password required: --password or KAPPA_PASSWORD", file=sys.stderr)
        return 1

    torch.manual_seed(SEED)
    if torch.cuda.is_available():
        torch.cuda.manual_seed(SEED)

    with KappaApkClient(args.base_url, args.login_id, args.password) as client:
        info = client.connect()
        print(f"Connected as {info.get('user_name', info.get('userName', '?'))}")

        dataset_id: int | None = None
        try:
            ds = client.get_dataset_details(dataset_name=args.dataset_name)
            dataset_id = ds.dataset_id
            print(f"Dataset '{args.dataset_name}' id={dataset_id}")
        except Exception as exc:
            print(f"Warning: could not resolve dataset id: {exc}")

        init_label_index(client, dataset_id)
        print(f"Classes ({NUM_CLASSES}): {', '.join(_CLASS_NAMES or FashionMNIST.classes)}")

        loader = make_kappa_loader(
            client,
            args.dataset_name,
            args.version_no,
            args.batch_size,
        )
        model = FashionMNISTCNN(NUM_CLASSES).to(DEVICE)
        optimizer = torch.optim.Adam(model.parameters(), lr=args.lr)
        criterion = nn.CrossEntropyLoss()

        epoch_losses: list[float] = []
        epoch_accuracies: list[float] = []

        for epoch in range(args.epochs):
            avg_loss, accuracy = train_epoch(
                model,
                loader,
                optimizer,
                criterion,
                train_only=args.train_only,
                max_batches=args.max_batches,
            )
            epoch_losses.append(avg_loss)
            epoch_accuracies.append(accuracy)
            print(
                f"Epoch {epoch + 1}/{args.epochs}  "
                f"loss={avg_loss:.4f}  acc={accuracy * 100:.2f}%  device={DEVICE}"
            )

        plot_path, metrics_path = save_curves(epoch_losses, epoch_accuracies, args.dataset_name)
        print(f"\nCurves: {plot_path}")
        print(f"Metrics: {metrics_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
