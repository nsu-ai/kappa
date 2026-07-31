#!/usr/bin/env python3
# Copyright 2026 Kappa-Apk @nsu.ru
# SPDX-License-Identifier: Apache-2.0
"""Dataset lifecycle ops example (Kappa-framework ≥ 2.10.0).

Covers flows beyond create/upload:

1. Filter / list datasets
2. Custom schema get (optional put)
3. Mark entities labeled
4. Soft-delete + recover entities
5. Version refresh / recover notes
6. Download entity file (optional)
7. Soft-delete + recover dataset (opt-in; destructive)

Use after ``dataset_operations_example.py`` has created a dataset with entities.
Bulk upload: ``bulk_upload.py``.

Usage::

    export KAPPA_URL=http://127.0.0.1:8060
    export KAPPA_USER=admin
    export KAPPA_PASSWORD='***'
    python code_examples/dataset_lifecycle_example.py --dataset-id 42
    python code_examples/dataset_lifecycle_example.py --dataset-name apk-demo-dataset \\
      --mark-labeled --soft-delete-entity --recover-entity
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

from kappa_apk import KappaApkClient, compatibility_info, min_backend_version, version


def _env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise SystemExit(f"Set environment variable {name}")
    return value


def _first_entity_id(page) -> str | None:
    if not isinstance(page, dict):
        return None
    items = page.get("items") or page.get("content") or page.get("data") or []
    if not items and isinstance(page.get("payload"), dict):
        items = page["payload"].get("items") or []
    if not isinstance(items, list) or not items:
        return None
    first = items[0]
    if isinstance(first, dict):
        return (
            first.get("dsEntityId")
            or first.get("ds_entity_id")
            or first.get("entityId")
            or first.get("id")
        )
    return getattr(first, "ds_entity_id", None) or getattr(first, "id", None)


def _first_file_id(entity) -> str | None:
    if not isinstance(entity, dict):
        return None
    files = (
        entity.get("files")
        or entity.get("entityFiles")
        or entity.get("datasetEntityFiles")
        or []
    )
    if isinstance(entity.get("payload"), dict) and not files:
        files = entity["payload"].get("files") or []
    if not isinstance(files, list) or not files:
        return None
    f0 = files[0]
    if isinstance(f0, dict):
        return f0.get("fileId") or f0.get("file_id") or f0.get("id")
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description="Kappa dataset lifecycle demo")
    parser.add_argument("--dataset-id", type=int, default=None)
    parser.add_argument("--dataset-name", default=None)
    parser.add_argument("--entity-id", default=None, help="Entity UUID (else first from filter)")
    parser.add_argument("--mark-labeled", action="store_true")
    parser.add_argument("--soft-delete-entity", action="store_true")
    parser.add_argument("--recover-entity", action="store_true")
    parser.add_argument("--refresh-version", default=None, help="version_no to refresh")
    parser.add_argument("--recover-version", default=None, help="version_no to recover")
    parser.add_argument(
        "--download-entity-file",
        default=None,
        metavar="DEST",
        help="Download first entity file to DEST path",
    )
    parser.add_argument(
        "--put-schema-json",
        default=None,
        help='Optional JSON object for put_dataset_custom_schema, e.g. \'{"columns":[]}\'',
    )
    parser.add_argument(
        "--soft-delete-dataset",
        action="store_true",
        help="Soft-delete the dataset (use --recover-dataset to undo)",
    )
    parser.add_argument("--recover-dataset", action="store_true")
    args = parser.parse_args()

    print("SDK", version(), "| min backend", min_backend_version())
    print("compat:", compatibility_info())

    base = _env("KAPPA_URL")
    user = _env("KAPPA_USER")
    password = _env("KAPPA_PASSWORD")

    with KappaApkClient(base, user, password) as client:
        # --- resolve dataset ---
        dataset_id = args.dataset_id
        dataset_name = args.dataset_name
        if dataset_id is None and dataset_name:
            ds = client.get_dataset_details(dataset_name=dataset_name)
            dataset_id = ds.dataset_id
            dataset_name = ds.dataset_name
        elif dataset_id is not None and not dataset_name:
            ds = client.get_dataset_details(dataset_id=dataset_id)
            dataset_name = getattr(ds, "dataset_name", None)
        if dataset_id is None:
            print("Pass --dataset-id or --dataset-name", file=sys.stderr)
            return 2

        print(f"Dataset id={dataset_id} name={dataset_name}")

        # --- filter / list ---
        listed = client.list_datasets(page=1, size=5, order_by="modifiedOn", order_keyword="DESC")
        print("list_datasets sample keys:", list(listed.keys()) if isinstance(listed, dict) else type(listed))
        filtered = client.filter_datasets(search=dataset_name or "", page=1, size=5)
        print("filter_datasets:", type(filtered).__name__)

        # --- custom schema ---
        try:
            schema = client.get_dataset_custom_schema(dataset_id)
            print("custom schema:", schema)
        except Exception as exc:
            print(f"get_dataset_custom_schema: {exc}")
        if args.put_schema_json:
            import json

            put = client.put_dataset_custom_schema(dataset_id, json.loads(args.put_schema_json))
            print("put_dataset_custom_schema:", put)

        # --- entities ---
        page = client.filter_dataset_entities(dataset_id, page=0, size=5)
        entity_id = args.entity_id or _first_entity_id(page)
        print("entity_id:", entity_id)

        if entity_id and args.mark_labeled:
            marked = client.mark_dataset_entities_labeled(
                dataset_id, [str(entity_id)], remark="apk lifecycle demo"
            )
            print("mark_dataset_entities_labeled:", marked)

        if entity_id and args.soft_delete_entity:
            deleted = client.delete_dataset_entities(
                [str(entity_id)], remark="apk lifecycle soft-delete"
            )
            print("delete_dataset_entities:", deleted)

        if entity_id and args.recover_entity:
            recovered = client.recover_dataset_entities([str(entity_id)])
            print("recover_dataset_entities:", recovered)

        if entity_id and args.download_entity_file:
            entity = client.get_dataset_entity(dataset_id, str(entity_id))
            file_id = _first_file_id(entity if isinstance(entity, dict) else {})
            if not file_id:
                print("No file_id on entity; skip download")
            else:
                dest = Path(args.download_entity_file)
                dest.parent.mkdir(parents=True, exist_ok=True)
                written = client.download_dataset_entity_file(
                    dataset_id, str(file_id), str(dest)
                )
                print("download_dataset_entity_file:", written)

        # --- versions ---
        versions = client.list_dataset_versions(dataset_id)
        print("versions:", versions)
        if args.refresh_version:
            print(
                "refresh_dataset_version:",
                client.refresh_dataset_version(dataset_id, args.refresh_version),
            )
        if args.recover_version:
            print(
                "recover_dataset_version:",
                client.recover_dataset_version(dataset_id, args.recover_version),
            )

        # --- dataset soft-delete / recover (opt-in) ---
        if args.soft_delete_dataset:
            print(
                "delete_dataset:",
                client.delete_dataset(dataset_id, remark="apk lifecycle soft-delete"),
            )
        if args.recover_dataset:
            print("recover_datasets:", client.recover_datasets([dataset_id]))

        print(
            "\nRelated examples:\n"
            "  code_examples/dataset_operations_example.py  # create → labels → entity → version\n"
            "  code_examples/bulk_upload.py                 # archive/CSV bulk + job poll\n"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
