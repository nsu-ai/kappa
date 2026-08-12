#!/usr/bin/env python3
# Copyright 2026 Kappa-Apk @nsu.ru
# SPDX-License-Identifier: Apache-2.0
"""Benchmark run + model artifact example (Kappa-framework ≥ 2.11.0).

Demonstrates:

1. Benchmark lookup and evaluation-set download (package → proxy → legacy zip)
2. Submitting predictions, uploading the model files, and linking the inference
3. Report metadata + PDF download
4. Writing an inference straight from predictions with ``write_model_inference``
5. Artifact upload (large files switch to a resumable multipart session) and package download

Usage::

    export KAPPA_URL=http://127.0.0.1:8060
    export KAPPA_USER=admin
    export KAPPA_PASSWORD='***'
    python code_examples/benchmark_and_model_artifacts.py --benchmark-id <uuid> --run \
        --model-path ./model
    python code_examples/benchmark_and_model_artifacts.py --benchmark-id <uuid> --report report.pdf
    python code_examples/benchmark_and_model_artifacts.py --model-id <uuid> \
        --write-inference --model-path ./model
    python code_examples/benchmark_and_model_artifacts.py \
        --model-id <uuid> --inference-id 12 --upload ./llm-70b --file-category 3
"""

from __future__ import annotations

import argparse
import os
from typing import Any, Optional

from kappa_apk import KappaApkClient, min_backend_version, version


def _env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise SystemExit(f"Set environment variable {name}")
    return value


def on_file_progress(name: str, sent: int, total: int, percent: int) -> None:
    print(f"  {name}: {percent}% ({sent}/{total} bytes)")


def run_benchmark(
    client: KappaApkClient,
    benchmark_id: str,
    model_path: Optional[str],
) -> None:
    detail = client.get_benchmark(benchmark_id)
    print(
        "benchmark", benchmark_id,
        "| status", detail.get("benchmarkStatus"),
        "| dataset", detail.get("datasetId"), detail.get("datasetVersionNo"),
    )

    bm = client.load_benchmark(benchmark_id)
    try:
        data = bm.dataset()
    except RuntimeError as exc:
        # The version archive build has not reached buildStatus=ready yet.
        print("evaluation set not ready:", exc)
        return
    except PermissionError as exc:
        print("download not permitted (approval may be pending):", exc)
        return
    print("loaded", len(data), "samples")

    predictions = [
        {"entity_id": item.entity_id, "original": item.annotations,
         "predicted": {"class_name": "unknown", "confidence": 0.0}}
        for item in data
    ]
    bm.save_benchmark(predictions, {"accuracy": 0.0}, model_path=model_path)
    # complete_inference (default) also links the inference to the benchmark,
    # which is what moves it from Pending Inference to Inference Completed.
    # upload_artifacts stores the files under model_path on that inference.
    response = bm.submit_benchmark(
        strict=False,
        upload_artifacts=bool(model_path),
        on_progress=on_file_progress,
    )
    print("submitted:", response)


def write_inference(
    client: KappaApkClient,
    model_id: str,
    model_path: Optional[str],
) -> None:
    """One call: shape the result per the model's schema, validate, create, upload."""
    schema = client.get_model_inference_schema(model_id)
    print("effective schema:", schema.get("source"), schema.get("kind"))

    written = client.write_model_inference(
        model_id,
        predictions=[
            {"entityId": "sample-1", "predicted": {"class_name": "pizza", "confidence": 0.98}},
            # A bare label works too — it is wrapped into the key the schema requires.
            {"entityId": "sample-2", "predicted": "sushi"},
        ],
        metrics={"accuracy": 0.93},
        artifacts=[model_path] if model_path else None,
        on_progress=on_file_progress,
    )
    print("inference", written["inferenceId"], "artifacts:", len(written["artifacts"]))


def show_report(client: KappaApkClient, benchmark_id: str, dest_path: str, lang: str) -> None:
    report = client.get_benchmark_report(benchmark_id)
    print("report:", report)
    path = client.download_benchmark_report(benchmark_id, dest_path, lang=lang)
    print("report written to", path)


def upload_artifact(
    client: KappaApkClient,
    model_id: str,
    inference_id: int,
    path: str,
    file_category: Optional[int],
) -> None:
    """Upload a file or a whole directory of weight shards.

    Each file picks its own transport: the plain upload for sidecars, a resumable
    multipart session past the server's sync cap. Re-running skips what is already
    attached, so an interrupted 100 GB upload can simply be started again.
    """
    print(f"uploading {path}")
    results: list[dict[str, Any]] = client.upload_model_artifacts(
        model_id, inference_id, path,
        file_category=file_category,
        on_progress=on_file_progress,
    )
    for item in results:
        print(f"  {item['fileName']}: {item['transport']} ({item['bytes']} bytes)")


def download_artifacts(
    client: KappaApkClient,
    model_id: str,
    inference_id: int,
    dest_dir: str,
) -> None:
    manifest = client.get_model_inference_artifacts_package(model_id, inference_id)
    print(
        "artifacts:", manifest.get("fileCount"),
        "files,", manifest.get("totalBytes"), "bytes",
    )
    paths = client.download_model_inference_artifacts_package(
        model_id, inference_id, dest_dir,
    )
    for path in paths:
        print("  ", path)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--benchmark-id")
    parser.add_argument("--run", action="store_true", help="download set, submit predictions")
    parser.add_argument("--report", help="write the benchmark report PDF to this path")
    parser.add_argument("--report-lang", default="en", choices=["en", "ru"])
    parser.add_argument("--model-id")
    parser.add_argument("--inference-id", type=int)
    parser.add_argument("--model-path", help="directory with the model files to upload")
    parser.add_argument(
        "--write-inference",
        action="store_true",
        help="write an inference (and --model-path artifacts) for --model-id",
    )
    parser.add_argument("--upload", help="artifact file or directory to upload")
    parser.add_argument("--file-category", type=int, help="1 Training 2 Inference 3 Model 4 Data 5 Other")
    parser.add_argument("--download-artifacts", help="directory for per-file artifact download")
    args = parser.parse_args()

    print("SDK", version(), "| min backend", min_backend_version())

    client = KappaApkClient(_env("KAPPA_URL"), _env("KAPPA_USER"), _env("KAPPA_PASSWORD"))
    client.connect()
    try:
        if args.benchmark_id and args.run:
            run_benchmark(client, args.benchmark_id, args.model_path)
        if args.benchmark_id and args.report:
            show_report(client, args.benchmark_id, args.report, args.report_lang)
        if args.model_id and args.write_inference:
            write_inference(client, args.model_id, args.model_path)

        artifacts_target = args.model_id and args.inference_id is not None
        if artifacts_target and args.upload:
            upload_artifact(
                client, args.model_id, args.inference_id, args.upload, args.file_category,
            )
        if artifacts_target and args.download_artifacts:
            download_artifacts(
                client, args.model_id, args.inference_id, args.download_artifacts,
            )
    finally:
        client.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
