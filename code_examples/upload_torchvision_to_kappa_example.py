#!/usr/bin/env python3
# Copyright 2026 Kappa-Apk @nsu.ru
# SPDX-License-Identifier: Apache-2.0
"""
Export Fashion-MNIST or CIFAR-100 from torchvision and upload to a local Kappa instance.

Materializes images under ``code_examples/data/<dataset>/train|test/<class>/*.png``,
then creates dataset labels and entities with ``labeling_algo=default`` (same flow as
``upload_pizza_steak_sushi_example.py``).

Setup::

    ./code_examples/setup_venv.sh
    source code_examples/.venv/bin/activate

Examples::

    # Fashion-MNIST (10 classes) — smoke test
    python upload_torchvision_to_kappa_example.py --dataset fashion_mnist \\
        --base-url http://127.0.0.1:8060 --login-id admin --password '***' \\
        --limit-per-class 20

    # CIFAR-100 (100 fine labels) — smoke test
    python upload_torchvision_to_kappa_example.py --dataset cifar100 \\
        --limit-per-class 5 --limit 500

    # Export only (no API)
    python upload_torchvision_to_kappa_example.py --dataset fashion_mnist --export-only

Full datasets (slow): omit ``--limit`` / ``--limit-per-class`` or set ``--limit-per-class 0``.

Environment: KAPPA_BASE_URL, KAPPA_LOGIN_ID, KAPPA_PASSWORD.
"""

from __future__ import annotations

import argparse
import os
import shutil
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable

try:
    from torchvision import transforms
    from torchvision.datasets import CIFAR100, FashionMNIST
except ImportError as e:
    print("Missing torchvision. Install: pip install torch torchvision", file=sys.stderr)
    raise SystemExit(1) from e

from upload_kappa_helpers import (
    ensure_labels,
    get_or_create_dataset,
    import_kappa,
    iter_exported_images,
    maybe_version_and_publish,
    safe_class_dir,
    upload_exported_images,
)

SCRIPT_DIR = Path(__file__).resolve().parent
DATA_DIR = SCRIPT_DIR / "data"
TO_PIL = transforms.ToPILImage()


@dataclass(frozen=True)
class DatasetSpec:
    key: str
    default_kappa_name: str
    short_info: str
    tags: str
    export_subdir: str
    class_names_fn: Callable[[], tuple[str, ...]]
    build_train: Callable[[Path], Any]
    build_test: Callable[[Path], Any]

    def class_names(self) -> tuple[str, ...]:
        return self.class_names_fn()


def _fashion_mnist_classes() -> tuple[str, ...]:
    return tuple(FashionMNIST.classes)


def _cifar100_classes() -> tuple[str, ...]:
    tmp = CIFAR100(root=str(DATA_DIR / "_cifar_meta"), download=True)
    return tuple(tmp.classes)


def _build_fashion_mnist(root: Path, train: bool):
    return FashionMNIST(root=str(root), train=train, download=True)


def _build_cifar100(root: Path, train: bool):
    return CIFAR100(root=str(root), train=train, download=True)


DATASETS: dict[str, DatasetSpec] = {
    "fashion_mnist": DatasetSpec(
        key="fashion_mnist",
        default_kappa_name="FashionMNIST",
        short_info="Fashion-MNIST (10 classes, 28x28 grayscale) via torchvision",
        tags="vision,classification,fashion-mnist,torchvision",
        export_subdir="fashion_mnist",
        class_names_fn=_fashion_mnist_classes,
        build_train=lambda r: _build_fashion_mnist(r, True),
        build_test=lambda r: _build_fashion_mnist(r, False),
    ),
    "cifar100": DatasetSpec(
        key="cifar100",
        default_kappa_name="CIFAR100",
        short_info="CIFAR-100 fine labels (100 classes, 32x32 RGB) via torchvision",
        tags="vision,classification,cifar-100,torchvision",
        export_subdir="cifar100",
        class_names_fn=_cifar100_classes,
        build_train=lambda r: _build_cifar100(r, True),
        build_test=lambda r: _build_cifar100(r, False),
    ),
}


def to_pil_image(image: Any) -> Any:
    if hasattr(image, "save"):
        return image
    return TO_PIL(image)


def export_split(
    spec: DatasetSpec,
    export_root: Path,
    split: str,
    train: bool,
    limit_per_class: int | None,
    download_root: Path,
) -> int:
    """Write PNGs to export_root/split/<class>/ and return count written."""
    ds = spec.build_train(download_root) if train else spec.build_test(download_root)
    names = spec.class_names()
    n_classes = len(names)
    per_class: dict[int, int] = defaultdict(int)
    written = 0
    split_dir = export_root / split
    split_dir.mkdir(parents=True, exist_ok=True)

    for idx in range(len(ds)):
        if limit_per_class is not None and len(per_class) >= n_classes:
            if all(per_class.get(i, 0) >= limit_per_class for i in range(n_classes)):
                break
        image, label_idx = ds[idx]
        if limit_per_class is not None and per_class[label_idx] >= limit_per_class:
            continue
        class_name = safe_class_dir(names[label_idx])
        class_dir = split_dir / class_name
        class_dir.mkdir(parents=True, exist_ok=True)
        out_path = class_dir / f"{idx:06d}.png"
        to_pil_image(image).save(out_path)
        per_class[label_idx] += 1
        written += 1

    print(f"  Exported {written} images -> {split_dir} ({len(per_class)} classes)")
    return written


def export_dataset(
    spec: DatasetSpec,
    splits: list[str],
    limit_per_class: int | None,
    force_export: bool,
) -> Path:
    export_root = DATA_DIR / spec.export_subdir
    download_root = DATA_DIR / f"{spec.export_subdir}_torchvision"

    meta_path = export_root / "_meta.json"
    cached_limit: int | None = None
    meta_present = meta_path.exists()
    if meta_present:
        try:
            meta = json.loads(meta_path.read_text(encoding="utf-8"))
            cached_limit = meta.get("limit_per_class")
        except Exception:
            meta_present = False

    if force_export and export_root.exists():
        shutil.rmtree(export_root, ignore_errors=True)

    ready = export_root.exists() and any((export_root / s).is_dir() for s in splits)
    if ready and not force_export:
        # Important: cache must be compatible with the requested per-class cap.
        # Otherwise users see “limit-per-class=100 but only 40 uploaded”.
        if not meta_present:
            print(
                f"No cache metadata found at {meta_path}. Refusing to reuse stale export. "
                f"Re-exporting to match limit-per-class={limit_per_class}."
            )
        elif cached_limit != limit_per_class:
            print(
                f"Cached export at {export_root} was created with limit-per-class={cached_limit}, "
                f"but you requested limit-per-class={limit_per_class}. Re-exporting (or use --force-export)."
            )
        else:
            print(f"Using cached export at {export_root}")
            return export_root

    print(f"Exporting {spec.key} to {export_root} (download cache: {download_root})")
    total = 0
    if "train" in splits:
        total += export_split(spec, export_root, "train", True, limit_per_class, download_root)
    if "test" in splits:
        total += export_split(spec, export_root, "test", False, limit_per_class, download_root)
    print(f"Export complete: {total} images under {export_root}")

    export_root.mkdir(parents=True, exist_ok=True)
    try:
        meta_path.write_text(
            json.dumps(
                {
                    "dataset_key": spec.key,
                    "limit_per_class": limit_per_class,
                    "splits": splits,
                },
                indent=2,
            ),
            encoding="utf-8",
        )
    except Exception:
        pass
    return export_root


def count_exported(root: Path, splits: list[str], limit: int | None) -> dict[str, int]:
    counts: dict[str, int] = {}
    for idx, (split, _, _) in enumerate(iter_exported_images(root, splits)):
        if limit is not None and idx >= limit:
            break
        counts[split] = counts.get(split, 0) + 1
    return counts


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Export Fashion-MNIST or CIFAR-100 and upload to Kappa.",
    )
    parser.add_argument(
        "--dataset",
        choices=tuple(DATASETS.keys()),
        required=True,
        help="torchvision dataset to export and upload",
    )
    parser.add_argument(
        "--base-url",
        default=os.environ.get("KAPPA_BASE_URL", "http://127.0.0.1:8060"),
    )
    parser.add_argument("--login-id", default=os.environ.get("KAPPA_LOGIN_ID", "admin"))
    parser.add_argument("--password", default=os.environ.get("KAPPA_PASSWORD", ""))
    parser.add_argument(
        "--dataset-name",
        default=None,
        help="Kappa dataset name (default: FashionMNIST or CIFAR100)",
    )
    parser.add_argument(
        "--splits",
        default="train,test",
        help="Splits to export/upload (default: train,test)",
    )
    parser.add_argument(
        "--labeling-algo",
        default="default",
        help="Kappa labelingAlgo per entity (default: default)",
    )
    parser.add_argument(
        "--limit-per-class",
        type=int,
        default=50,
        help="Max images per class per split (default: 50). Use 0 for no cap.",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Stop after N total uploads (after per-class cap)",
    )
    parser.add_argument("--no-reuse-dataset", action="store_true")
    parser.add_argument(
        "--force-export",
        action="store_true",
        help="Re-export PNGs even if cache exists",
    )
    parser.add_argument("--export-only", action="store_true", help="Export PNGs only")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--create-version", action="store_true")
    parser.add_argument("--publish", action="store_true")
    parser.add_argument("--version-no", default="1.0.0")
    parser.add_argument("--publish-type", type=int, default=0, choices=(0, 1, 2))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    spec = DATASETS[args.dataset]
    kappa_name = args.dataset_name or spec.default_kappa_name
    splits = [s.strip() for s in args.splits.split(",") if s.strip()]
    if not splits:
        print("No splits selected", file=sys.stderr)
        return 1

    per_class_cap: int | None
    if args.limit_per_class <= 0:
        per_class_cap = None
        print("No per-class limit — exporting full split(s) (may take a long time).")
    else:
        per_class_cap = args.limit_per_class

    root = export_dataset(spec, splits, per_class_cap, args.force_export)
    names = spec.class_names()
    class_dirs = sorted({safe_class_dir(c) for c in names})
    by_split = count_exported(root, splits, args.limit)
    total = sum(by_split.values())
    print(
        f"Ready: {total} images, {len(class_dirs)} classes, "
        f"splits: {', '.join(f'{k}={v}' for k, v in sorted(by_split.items()))}"
    )

    if args.export_only:
        return 0

    if not args.password and not args.dry_run:
        print("Password required: --password or KAPPA_PASSWORD", file=sys.stderr)
        return 1

    if args.dry_run:
        ok, _, uploaded = upload_exported_images(
            None,
            0,
            root,
            splits,
            class_dirs,
            spec.export_subdir,
            args.labeling_algo,
            dry_run=True,
            limit=args.limit,
        )
        print(f"Dry-run complete ({ok} files: {uploaded})")
        return 0

    KappaApkClient, NewDataset, NewDatasetEntity, NewDatasetVersion = import_kappa()

    with KappaApkClient(args.base_url, args.login_id, args.password) as client:
        info = client.connect()
        print(f"Connected as {info.get('user_name', info.get('userName', '?'))}")

        dataset_id = get_or_create_dataset(
            client,
            kappa_name,
            spec.short_info,
            spec.tags,
            not args.no_reuse_dataset,
            NewDataset,
        )
        ensure_labels(client, dataset_id, list(names))
        ok, failed, uploaded = upload_exported_images(
            client,
            dataset_id,
            root,
            splits,
            class_dirs,
            spec.export_subdir,
            args.labeling_algo,
            dry_run=False,
            limit=args.limit,
            NewDatasetEntity=NewDatasetEntity,
        )
        uploaded_summary = ", ".join(f"{k}={v}" for k, v in sorted(uploaded.items()))
        print(
            f"Done: {ok} uploaded ({uploaded_summary}), {failed} failed "
            f"(dataset_id={dataset_id}, name={kappa_name})"
        )

        if args.create_version or args.publish:
            maybe_version_and_publish(
                client,
                dataset_id,
                create_version=args.create_version,
                version_remark=f"Uploaded via upload_torchvision_to_kappa ({spec.key})",
                publish=args.publish,
                publish_type=args.publish_type,
                version_no=args.version_no,
                NewDatasetVersion=NewDatasetVersion,
            )

        if failed:
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
