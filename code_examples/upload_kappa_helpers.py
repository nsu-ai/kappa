# Copyright 2026 Kappa-Apk @nsu.ru
# SPDX-License-Identifier: Apache-2.0
"""Shared helpers for uploading vision datasets to Kappa."""

from __future__ import annotations

import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterator


def dig(mapping: object, *keys: str) -> object | None:
    cur = mapping
    for key in keys:
        if not isinstance(cur, dict):
            return None
        cur = cur.get(key)
    return cur


def dataset_id_from_response(resp: object) -> int | None:
    if not isinstance(resp, dict):
        return None
    for path in (("data", "datasetId"), ("data", "dataset_id"), ("datasetId",)):
        value = dig(resp, *path)
        if value is not None:
            return int(value)
    return None


def lookup_dataset_id(client: Any, dataset_name: str) -> int:
    ds = client.get_dataset_details(dataset_name=dataset_name)
    return ds.dataset_id


def api_response_ok(resp: object) -> bool:
    if isinstance(resp, str):
        low = resp.lower()
        return any(word in low for word in ("success", "added", "updated"))
    return True


def entity_id_from_response(resp: object) -> str | None:
    if not isinstance(resp, dict):
        return None
    for path in (
        ("data", "dsEntityId"),
        ("data", "ds_entity_id"),
        ("data", "entityId"),
        ("data", "entity_id"),
    ):
        value = dig(resp, *path)
        if value is not None:
            return str(value)
    return None


def safe_class_dir(name: str) -> str:
    cleaned = re.sub(r"[^\w\-.]+", "_", name.strip())
    return cleaned or "unknown"


def label_display(class_name: str) -> str:
    return class_name.replace("_", " ").strip()


def entity_info_for_default_label(class_name: str, split: str, label_id: int | None = None) -> dict:
    label = label_display(class_name)
    info: dict = {
        "split": split,
        "label": label,
        "annotations": [
            {
                "type": "choices",
                "value": {"choices": [label]},
                "to_name": "image",
                "from_name": "choice",
            }
        ],
    }
    if label_id is not None:
        info["label_id"] = label_id
    return info


def import_kappa():
    try:
        from kappa_apk import (
            KappaApkClient,
            NewDataset,
            NewDatasetEntity,
            NewDatasetVersion,
        )
    except ImportError as e:
        print(
            "Missing kappa_apk. Build and install the wheel, e.g.\n"
            "  ./code_examples/setup_venv.sh\n"
            "  source code_examples/.venv/bin/activate",
            file=sys.stderr,
        )
        raise SystemExit(1) from e
    return KappaApkClient, NewDataset, NewDatasetEntity, NewDatasetVersion


def get_or_create_dataset(
    client: Any,
    dataset_name: str,
    dataset_short_info: str,
    dataset_tags: str,
    reuse_existing: bool,
    NewDataset: Any,
) -> int:
    if reuse_existing:
        try:
            dataset_id = lookup_dataset_id(client, dataset_name)
            print(f"Reusing dataset '{dataset_name}' (id={dataset_id})")
            return dataset_id
        except Exception:
            pass

    payload = NewDataset(
        dataset_name=dataset_name,
        dataset_type=1,
        dataset_short_info=dataset_short_info,
        dataset_tags=dataset_tags,
        dataset_verification_type=1,
    )
    try:
        resp = client.add_dataset(payload)
        parsed_id = dataset_id_from_response(resp)
        if isinstance(resp, str):
            print(resp)
        elif parsed_id is not None:
            print(f"Created dataset '{dataset_name}' (id={parsed_id})")
            return parsed_id
    except Exception as exc:
        msg = str(exc).lower()
        if not any(token in msg for token in ("exist", "already", "duplicate")):
            raise
        print(f"Dataset '{dataset_name}' already exists ({exc}); looking up id")

    dataset_id = lookup_dataset_id(client, dataset_name)
    print(f"Using dataset '{dataset_name}' (id={dataset_id})")
    return dataset_id


def list_existing_label_names(client: Any, dataset_id: int) -> set[str]:
    try:
        return {name.lower() for name in client.get_dataset_label_names(dataset_id)}
    except ValueError as exc:
        msg = str(exc).lower()
        if "no_record" in msg:
            print("No labels on dataset yet; will add class labels")
            return set()
        raise


def ensure_labels(client: Any, dataset_id: int, class_names: list[str]) -> None:
    existing = list_existing_label_names(client, dataset_id)
    to_add = [
        label_display(c)
        for c in class_names
        if label_display(c).lower() not in existing
    ]
    if not to_add:
        print("Dataset labels already present (or subset already registered)")
        return
    resp = client.add_dataset_labels(dataset_id, to_add)
    if isinstance(resp, str):
        print(f"Labels: {resp}")
    else:
        print(f"Added {len(to_add)} labels (showing first 5): {to_add[:5]}...")


IMAGE_SUFFIXES = {".jpg", ".jpeg", ".png", ".bmp", ".webp", ".gif"}


def iter_exported_images(
    root: Path,
    splits: list[str],
    class_names: list[str] | None = None,
) -> Iterator[tuple[str, str, Path]]:
    """Yield (split, class_dir_name, path) from train/test/class layout."""
    for split in splits:
        split_dir = root / split
        if not split_dir.is_dir():
            raise SystemExit(f"Missing split directory: {split_dir}")
        class_dirs = sorted(
            p for p in split_dir.iterdir() if p.is_dir() and not p.name.startswith(".")
        )
        if class_names is not None:
            allowed = {safe_class_dir(c) for c in class_names}
            class_dirs = [p for p in class_dirs if p.name in allowed]
        for class_dir in class_dirs:
            for path in sorted(class_dir.iterdir()):
                if path.is_file() and path.suffix.lower() in IMAGE_SUFFIXES:
                    yield split, class_dir.name, path


def upload_exported_images(
    client: Any | None,
    dataset_id: int,
    root: Path,
    splits: list[str],
    class_names: list[str],
    source_prefix: str,
    labeling_algo: str,
    dry_run: bool,
    limit: int | None,
    NewDatasetEntity: Any | None = None,
) -> tuple[int, int, dict[str, int]]:
    collected_on = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    ok = 0
    failed = 0
    by_split: dict[str, int] = {}
    for idx, (split, class_name, image_path) in enumerate(
        iter_exported_images(root, splits, class_names)
    ):
        if limit is not None and idx >= limit:
            break
        entity_name = f"{split}_{class_name}_{image_path.stem}"
        info = entity_info_for_default_label(class_name, split)
        if dry_run:
            print(f"[dry-run] {entity_name} <- {image_path}")
            ok += 1
            by_split[split] = by_split.get(split, 0) + 1
            continue
        if NewDatasetEntity is None:
            raise RuntimeError("NewDatasetEntity required for upload")
        entity = NewDatasetEntity(
            ds_entity_name=entity_name,
            collected_on=collected_on,
            labeling_algo=labeling_algo,
            ds_entity_info=info,
            entity_source=f"{source_prefix}/{split}/{class_name}",
        )
        assert client is not None
        try:
            resp = client.add_dataset_entity(
                dataset_id,
                entity,
                file_paths=[str(image_path)],
            )
            if not api_response_ok(resp):
                raise RuntimeError(f"Unexpected entity response: {resp!r}")
            ok += 1
            by_split[split] = by_split.get(split, 0) + 1
            if ok % 100 == 0:
                print(f"  ... {ok} uploaded")
        except Exception as exc:
            failed += 1
            print(f"FAILED {entity_name}: {exc}", file=sys.stderr)
    return ok, failed, by_split


def maybe_version_and_publish(
    client: Any,
    dataset_id: int,
    create_version: bool,
    version_remark: str,
    publish: bool,
    publish_type: int,
    version_no: str,
    NewDatasetVersion: Any,
) -> None:
    if create_version:
        ver = NewDatasetVersion(version_availability=1, version_remark=version_remark)
        client.create_dataset_version(dataset_id, ver)
        print("Created dataset version")
    if publish:
        client.publish_dataset_version(dataset_id, version_no, publish_type)
        print(f"Published version {version_no} (publish_type={publish_type})")
