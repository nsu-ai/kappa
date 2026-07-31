#!/usr/bin/env python3
"""Bulk entity upload + job progress example (Kappa-framework ≥ 2.10.0).

Requires: ``pip install kf-sdk`` (or local maturin) and a dataset you can write to.

Usage::

    export KAPPA_URL=https://your-gateway/
    export KAPPA_USER=you@example.com
    export KAPPA_PASSWORD=secret
    python code_examples/bulk_upload.py --dataset-id 42 --file ./data.zip \\
        --upload-type archive --archive-layout input_output \\
        --input-data-path input
"""

from __future__ import annotations

import argparse
import json
import os
import sys

from kappa_apk import KappaApkClient, compatibility_info, min_backend_version, version


def main() -> int:
    parser = argparse.ArgumentParser(description="Kappa APK bulk upload demo")
    parser.add_argument("--dataset-id", type=int, required=True)
    parser.add_argument("--file", required=True, help="Path to .zip or .csv")
    parser.add_argument("--upload-type", choices=("archive", "csv"), required=True)
    parser.add_argument(
        "--archive-layout",
        choices=("input_output", "classes"),
        default=None,
        help="Required for archive uploads",
    )
    parser.add_argument(
        "--input-data-path",
        default="input",
        help="Zip folder for inputs when archive_layout=input_output",
    )
    parser.add_argument(
        "--output-data-path",
        default=None,
        help="Optional zip folder for outputs (input_output)",
    )
    parser.add_argument(
        "--dataset-schema-json",
        default=None,
        help="Optional full dataset_schema JSON (overrides path helpers)",
    )
    parser.add_argument("--labeling-algo", default="default")
    parser.add_argument("--source", default="apk-example")
    parser.add_argument("--bulk-split", default="train")
    parser.add_argument("--strict", action="store_true", default=True)
    parser.add_argument("--check-permission", action="store_true")
    args = parser.parse_args()

    print("SDK", version(), "| min backend", min_backend_version())
    print("compat:", compatibility_info())

    base = os.environ.get("KAPPA_URL")
    user = os.environ.get("KAPPA_USER")
    password = os.environ.get("KAPPA_PASSWORD")
    if not base or not user or not password:
        print("Set KAPPA_URL, KAPPA_USER, KAPPA_PASSWORD", file=sys.stderr)
        return 2

    dataset_schema = None
    if args.dataset_schema_json:
        dataset_schema = json.loads(args.dataset_schema_json)
    elif args.upload_type == "archive" and args.archive_layout == "input_output":
        dataset_schema = {"inputDataPath": args.input_data_path}
        if args.output_data_path:
            dataset_schema["outputDataPath"] = args.output_data_path
    elif args.upload_type == "archive" and args.archive_layout == "classes":
        print(
            "For archive_layout=classes pass --dataset-schema-json "
            '\'{"classes":[{"className":"cat","path":"cat","split":"train"}]}\'',
            file=sys.stderr,
        )
        return 2

    with KappaApkClient(base, user, password) as client:
        if args.check_permission:
            ok = client.has_permission("dataset.write", dataset_id=args.dataset_id)
            print("dataset.write?", ok)
            if not ok:
                print(client.get_my_permissions(dataset_id=args.dataset_id))
                return 1

        def on_upload(sent: int, total: int, percent: int) -> None:
            print(f"\rUpload transfer: {percent}% ({sent}/{total} bytes)", end="", flush=True)

        start = client.bulk_upload_dataset_entities(
            dataset_id=args.dataset_id,
            file_path=args.file,
            upload_type=args.upload_type,
            labeling_algo=args.labeling_algo,
            source=args.source,
            dataset_schema=dataset_schema,
            bulk_split=args.bulk_split if args.upload_type == "csv" else None,
            archive_layout=args.archive_layout,
            strict=args.strict,
            on_upload_progress=on_upload,
            check_permission=args.check_permission,
        )
        print()
        job_id = start.get("jobId") or start.get("job_id")
        print("Started job:", job_id, "status:", start.get("status"))

        def on_job(job) -> None:
            pct = job.percent if job.percent is not None else "…"
            print(
                f"Job {job.status} phase={job.phase} "
                f"{job.processed_rows}/{job.total_rows} ({pct}%)"
            )

        final = client.wait_for_bulk_upload_job(
            args.dataset_id,
            str(job_id),
            poll_interval_secs=2.0,
            timeout_secs=3600.0,
            on_progress=on_job,
        )
        print("Finished:", final)
        print("can_retry=", final.can_retry, "terminal=", final.is_terminal())
        if final.status.lower() in ("failed", "error"):
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
