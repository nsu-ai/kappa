#!/usr/bin/env python3
# Copyright 2026 Kappa-Apk @nsu.ru
# SPDX-License-Identifier: Apache-2.0
"""Bulk mutation + version build example (Kappa-framework ≥ 2.11.0).

Demonstrates:

1. Mark-labeled (``all_eligible`` or entity IDs) + wait
2. Optional self-verify / auto-verify
3. Create version → wait for archive build → publish / package download

Usage::

    export KAPPA_URL=http://127.0.0.1:8060
    export KAPPA_USER=admin
    export KAPPA_PASSWORD='***'
    python code_examples/bulk_mutation_and_version_build.py --dataset-id 42 --mark-labeled-all
    python code_examples/bulk_mutation_and_version_build.py --dataset-id 42 --create-version
"""

from __future__ import annotations

import argparse
import os
import sys

from kappa_apk import (
    KappaApkClient,
    NewDatasetVersion,
    compatibility_info,
    min_backend_version,
    version,
)


def _env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise SystemExit(f"Set environment variable {name}")
    return value


def _job_id(start) -> str | None:
    if isinstance(start, dict):
        return start.get("jobId") or start.get("job_id")
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dataset-id", type=int, required=True)
    parser.add_argument("--mark-labeled-all", action="store_true")
    parser.add_argument("--entity-ids", nargs="*", default=[])
    parser.add_argument("--self-verify-pass", action="store_true")
    parser.add_argument("--auto-verify", action="store_true")
    parser.add_argument("--create-version", action="store_true")
    parser.add_argument("--publish-type", type=int, default=1)
    parser.add_argument("--download-package", action="store_true")
    parser.add_argument("--timeout-secs", type=float, default=3600.0)
    args = parser.parse_args()

    print("SDK", version(), "| min backend", min_backend_version())
    print("compat:", compatibility_info())

    client = KappaApkClient(_env("KAPPA_URL"), _env("KAPPA_USER"), _env("KAPPA_PASSWORD"))
    client.connect()
    dataset_id = args.dataset_id

    def on_mut(job):
        print(
            f"  mutation {job.job_type} {job.status} {job.phase} "
            f"{job.percent}% {job.processed_count}/{job.total_count}"
        )

    def on_build(job):
        print(
            f"  build {job.version_no} {job.status} {job.phase} "
            f"{job.percent}% entities {job.processed_entities}/{job.total_entities}"
        )

    if args.mark_labeled_all or args.entity_ids:
        stats = client.get_mark_labeled_stats(dataset_id)
        print("mark-labeled-stats:", stats)
        start = client.mark_dataset_entities_labeled(
            dataset_id,
            dataset_entity_ids=args.entity_ids or None,
            remark="apk example",
            all_eligible=True if args.mark_labeled_all else None,
        )
        print("mark-labeled start:", start)
        jid = _job_id(start)
        if jid:
            final = client.wait_for_bulk_mutation_job(
                dataset_id,
                jid,
                timeout_secs=args.timeout_secs,
                on_progress=on_mut,
            )
            print("mark-labeled final:", final.status, final.as_dict())

    if args.self_verify_pass:
        print("self-verify-stats:", client.get_self_verify_stats(dataset_id))
        start = client.bulk_self_verify_dataset_entities(dataset_id, status=1)
        print("self-verify start:", start)
        jid = _job_id(start)
        if jid:
            final = client.wait_for_bulk_mutation_job(
                dataset_id,
                jid,
                timeout_secs=args.timeout_secs,
                on_progress=on_mut,
            )
            print("self-verify final:", final.status, final.error_detail)

    if args.auto_verify:
        start = client.auto_verify_dataset_entities(dataset_id)
        print("auto-verify start:", start)
        jid = _job_id(start)
        if jid:
            final = client.wait_for_bulk_mutation_job(
                dataset_id,
                jid,
                timeout_secs=args.timeout_secs,
                on_progress=on_mut,
            )
            print("auto-verify final:", final.status, final.error_detail)

    version_no = None
    if args.create_version:
        created = client.create_dataset_version(
            dataset_id,
            NewDatasetVersion(version_availability=1, version_remark="apk 2.11 example"),
        )
        print("create_dataset_version:", created)
        version_no = (
            created.get("versionNo") or created.get("version_no")
            if isinstance(created, dict)
            else None
        )
        jid = _job_id(created)
        if jid:
            build = client.wait_for_version_build_job(
                jid,
                timeout_secs=args.timeout_secs,
                on_progress=on_build,
            )
            print("build final:", build.status, build.is_ready(), build.as_dict())
            if not build.is_ready():
                print("Build did not complete successfully; skip publish/download", file=sys.stderr)
                return 1
        if version_no:
            pub = client.publish_dataset_version(dataset_id, version_no, args.publish_type)
            print("publish:", pub)

    if args.download_package:
        if not version_no:
            versions = client.list_dataset_versions(dataset_id)
            print("versions:", versions)
            raise SystemExit("Pass --create-version or set version via prior create")
        info = client.download_dataset_version_package(
            dataset_id=dataset_id, version_no=version_no
        )
        print("package cache:", info.data_path, info.download_status)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
