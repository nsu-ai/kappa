#!/usr/bin/env python3
# Copyright 2026 Kappa-Apk @nsu.ru
# SPDX-License-Identifier: Apache-2.0
"""Dataset operations flow example (Kappa-framework ≥ 2.10.0).

Walks through a typical scripting flow:

1. Auth + permission check
2. Create / look up a dataset
3. Add labels
4. Add a single entity (with ``file_category`` + ``split``)
5. Filter / list entities
6. Create a version and optionally publish
7. Soft-delete / recover notes

Bulk upload is covered separately in ``bulk_upload.py``.

Usage::

    ./code_examples/setup_venv.sh
    source code_examples/.venv/bin/activate
    export KAPPA_URL=http://127.0.0.1:8060
    export KAPPA_USER=admin
    export KAPPA_PASSWORD='***'

    # Dry run of client helpers (no API) is not supported — needs a live gateway.
    python code_examples/dataset_operations_example.py \\
      --dataset-name apk-demo-dataset \\
      --image ./path/to/sample.jpg
"""

from __future__ import annotations

import argparse
import os
import sys
from datetime import datetime, timezone

from kappa_apk import (
    KappaApkClient,
    NewDataset,
    NewDatasetEntity,
    NewDatasetVersion,
    UpdateDatasetEntity,
    UpdateDatasetRequest,
    compatibility_info,
    join_ml_tags,
    min_backend_version,
    version,
)


def _env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise SystemExit(f"Set environment variable {name}")
    return value


def main() -> int:
    parser = argparse.ArgumentParser(description="Kappa dataset operations flow demo")
    parser.add_argument("--dataset-name", default="apk-demo-dataset")
    parser.add_argument("--dataset-type", type=int, default=1, help="1=Vision (default)")
    parser.add_argument(
        "--primary-tag",
        default=None,
        help="Predefined ML tag (must be first). Default: Image Classification for type=1",
    )
    parser.add_argument("--image", default=None, help="Optional local image for entity upload")
    parser.add_argument("--label", default="demo")
    parser.add_argument("--publish", action="store_true", help="Publish version after create")
    parser.add_argument(
        "--publish-type",
        type=int,
        default=1,
        help="0 Not Published · 1 Private · 2 Open Source · 3 Public on Demand · 4 Purchase",
    )
    parser.add_argument("--skip-create", action="store_true", help="Reuse existing dataset by name")
    args = parser.parse_args()

    print("SDK", version(), "| min backend", min_backend_version())
    print("compat:", compatibility_info())

    base = _env("KAPPA_URL")
    user = _env("KAPPA_USER")
    password = _env("KAPPA_PASSWORD")

    with KappaApkClient(base, user, password) as client:
        profile = client.get_user_profile()
        print(f"Logged in as {profile.user_name} (id={profile.user_id})")

        primary_tag = args.primary_tag
        if not primary_tag:
            catalog = client.list_predefined_ml_tags(args.dataset_type)
            primary_tag = catalog[0] if catalog else "Image Classification"
            print(f"Using primary ML tag {primary_tag!r} from catalog ({len(catalog)} tags)")

        # --- 1) Resolve or create dataset ---
        dataset_id: int | None = None
        if args.skip_create:
            ds = client.get_dataset_details(dataset_name=args.dataset_name)
            dataset_id = ds.dataset_id
            print(f"Using existing dataset id={dataset_id} name={ds.dataset_name}")
        else:
            try:
                ds = client.get_dataset_details(dataset_name=args.dataset_name)
                dataset_id = ds.dataset_id
                print(f"Dataset already exists id={dataset_id}")
            except Exception:
                created = client.add_dataset(
                    NewDataset(
                        dataset_name=args.dataset_name,
                        dataset_type=args.dataset_type,
                        dataset_short_info="Created by dataset_operations_example.py",
                        dataset_tags=join_ml_tags(primary_tag, "apk", "demo"),
                        dataset_verification_type=1,
                    )
                )
                print("add_dataset response:", created)
                ds = client.get_dataset_details(dataset_name=args.dataset_name)
                dataset_id = ds.dataset_id
                print(f"Created dataset id={dataset_id}")

        assert dataset_id is not None

        # --- 2) RBAC ---
        can_write = client.has_permission("dataset.write", dataset_id=dataset_id)
        print(f"dataset.write? {can_write}")
        if not can_write:
            print(client.get_my_permissions(dataset_id=dataset_id))
            return 1

        # --- 3) Labels ---
        try:
            client.add_dataset_labels(dataset_id, [args.label])
            print(f"Added label {args.label!r}")
        except Exception as exc:
            print(f"add_dataset_labels (may already exist): {exc}")
        labels = client.get_dataset_label_names(dataset_id)
        print("Labels:", labels)

        # --- 4) Optional metadata update ---
        try:
            client.update_dataset(
                dataset_id,
                UpdateDatasetRequest(
                    dataset_short_info="Updated by dataset_operations_example.py",
                    remark="demo update",
                ),
            )
            print("Updated dataset metadata")
        except Exception as exc:
            print(f"update_dataset skipped: {exc}")

        # --- 5) Single entity ---
        collected = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S%z")
        # normalize +0000 -> +00:00 if needed; backend accepts aware ISO-ish strings
        if len(collected) >= 5 and collected[-5] in "+-" and ":" not in collected[-5:]:
            collected = collected[:-2] + ":" + collected[-2:]

        entity = NewDatasetEntity(
            ds_entity_name=f"demo_{datetime.now(timezone.utc).strftime('%H%M%S')}",
            collected_on=collected,
            labeling_algo="default",
            ds_entity_info={"label": args.label, "annotations": []},
            entity_source="apk-demo",
            split="train",
        )
        file_paths = [args.image] if args.image else None
        add_resp = client.add_dataset_entity(
            dataset_id,
            entity,
            file_paths=file_paths,
            file_category="input" if file_paths else None,
            split="train",
        )
        print("add_dataset_entity:", add_resp)
        entity_id = None
        if isinstance(add_resp, dict):
            entity_id = (
                add_resp.get("ds_entity_id")
                or add_resp.get("dsEntityId")
                or (add_resp.get("payload") or {}).get("dsEntityId")
            )
        print("entity_id:", entity_id)

        if entity_id:
            client.update_dataset_entity(
                dataset_id,
                str(entity_id),
                UpdateDatasetEntity(remark="demo touch", split="train"),
            )
            print("Updated entity")

        # --- 6) Filter entities ---
        page = client.filter_dataset_entities(dataset_id, page=0, size=10)
        print("filter_dataset_entities keys:", list(page.keys()) if isinstance(page, dict) else type(page))

        # --- 7) Version ---
        ver = client.create_dataset_version(
            dataset_id,
            NewDatasetVersion(version_availability=1, version_remark="apk demo version"),
        )
        print("create_dataset_version:", ver)
        versions = client.list_dataset_versions(dataset_id)
        print("versions:", versions)

        version_no = None
        if isinstance(ver, dict):
            version_no = ver.get("versionNo") or ver.get("version_no")
        if not version_no and isinstance(versions, list) and versions:
            first = versions[0]
            version_no = (
                first.get("versionNo")
                if isinstance(first, dict)
                else getattr(first, "version_no", None)
            )

        if args.publish and version_no:
            pub = client.publish_dataset_version(dataset_id, str(version_no), args.publish_type)
            print("publish_dataset_version:", pub)

        print(
            "\nNext steps:\n"
            "  code_examples/bulk_upload.py               # archive/CSV bulk + job poll\n"
            "  code_examples/dataset_lifecycle_example.py # schema, mark-labeled, recover\n"
            "  (archive needs dataset_schema.inputDataPath or classes[])"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
