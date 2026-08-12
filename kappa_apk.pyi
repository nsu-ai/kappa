# Type stubs for kappa_apk — auto-maintained alongside src/
# Covers all Python-accessible classes, methods, and properties.

from __future__ import annotations
from typing import Any, Iterator, Optional

def version() -> str:
    """Return the library version string (kf-sdk / kappa_apk)."""
    ...

def min_backend_version() -> str:
    """Minimum Kappa-framework product version required (``\"2.11.0\"``)."""
    ...

def compatibility_info() -> dict[str, Any]:
    """``{sdk_version, min_backend_version, notes, …}`` for scripts and CI."""
    ...

def join_ml_tags(primary_ml_tag: str, *extras: str) -> str:
    """Join a predefined primary ML tag (first) with optional custom tags."""
    ...

def ensure_primary_ml_tag_first(tags: str, primary_ml_tag: str) -> str:
    """Move or insert ``primary_ml_tag`` as the first comma-separated tag."""
    ...

def validate_ml_tags(
    tags: str,
    predefined_display_values: list[str],
    require_primary_first: bool = True,
) -> None:
    """Raise ``ValueError`` if tags miss a predefined entry or primary is not first."""
    ...

# ---------------------------------------------------------------------------
# Data models — returned by client/benchmark methods
# ---------------------------------------------------------------------------

class ItemFile:
    """A single file belonging to a DatasetItem."""

    @property
    def file_id(self) -> str: ...
    @property
    def file_name(self) -> str: ...
    @property
    def file(self) -> str:
        """Absolute path to the file on disk."""
        ...

class DatasetItem:
    """One labelled sample in a dataset version."""

    @property
    def entity_id(self) -> str: ...
    @property
    def files(self) -> Optional[list[ItemFile]]: ...
    @property
    def annotations(self) -> Optional[list[dict[str, Any]]]: ...
    @property
    def entity_info(self) -> Optional[dict[str, Any]]: ...
    @property
    def split(self) -> str:
        """``dsEntityInfo.split``; defaults to ``\"train\"`` when absent."""
        ...

class BulkUploadJob:
    """Status snapshot for an async bulk entity upload job."""

    @property
    def job_id(self) -> str: ...
    @property
    def dataset_id(self) -> int: ...
    @property
    def upload_type(self) -> str: ...
    @property
    def status(self) -> str: ...
    @property
    def phase(self) -> Optional[str]: ...
    @property
    def total_rows(self) -> int: ...
    @property
    def processed_rows(self) -> int: ...
    @property
    def percent(self) -> Optional[int]:
        """Job processing percent from ``processedRows/totalRows``, or None."""
        ...
    @property
    def source(self) -> Optional[str]: ...
    @property
    def filename(self) -> Optional[str]: ...
    @property
    def failure_kind(self) -> Optional[str]: ...
    @property
    def labeling_algo(self) -> Optional[str]: ...
    @property
    def retryable(self) -> bool: ...
    @property
    def can_cancel(self) -> bool: ...
    @property
    def can_retry(self) -> bool: ...
    @property
    def is_stale(self) -> bool: ...
    @property
    def preflight_deferred(self) -> bool: ...
    def is_terminal(self) -> bool:
        """True for completed / failed / cancelled (FE ``isBulkJobTerminal``)."""
        ...

    def is_wait_complete(self) -> bool:
        """True when :meth:`wait_for_bulk_upload_job` should stop (terminal or needs_correction)."""
        ...

    def needs_correction(self) -> bool:
        """True when status is ``needs_correction`` (fix layout and retry)."""
        ...

    def as_dict(self) -> dict[str, Any]: ...

class BulkMutationJob:
    """Status snapshot for an async bulk mutation job (self-verify, mark-labeled, …)."""

    @property
    def job_id(self) -> str: ...
    @property
    def dataset_id(self) -> int: ...
    @property
    def job_type(self) -> str: ...
    @property
    def status(self) -> str: ...
    @property
    def phase(self) -> Optional[str]: ...
    @property
    def total_count(self) -> int: ...
    @property
    def processed_count(self) -> int: ...
    @property
    def succeeded_count(self) -> int: ...
    @property
    def skipped_count(self) -> int: ...
    @property
    def failed_count(self) -> int: ...
    @property
    def percent(self) -> Optional[int]: ...
    @property
    def eta_human(self) -> Optional[str]: ...
    @property
    def error_detail(self) -> Optional[str]: ...
    @property
    def can_cancel(self) -> bool: ...
    def is_terminal(self) -> bool: ...
    def is_wait_complete(self) -> bool: ...
    def as_dict(self) -> dict[str, Any]: ...

class VersionBuildJob:
    """Status snapshot for a dataset version archive build job."""

    @property
    def job_id(self) -> str: ...
    @property
    def dataset_id(self) -> int: ...
    @property
    def version_id(self) -> int: ...
    @property
    def version_no(self) -> str: ...
    @property
    def job_type(self) -> str: ...
    @property
    def status(self) -> str: ...
    @property
    def phase(self) -> Optional[str]: ...
    @property
    def total_entities(self) -> int: ...
    @property
    def processed_entities(self) -> int: ...
    @property
    def shards_total(self) -> int: ...
    @property
    def shards_uploaded(self) -> int: ...
    @property
    def percent(self) -> Optional[int]: ...
    @property
    def eta_human(self) -> Optional[str]: ...
    @property
    def error_json(self) -> Optional[str]: ...
    def is_terminal(self) -> bool: ...
    def is_wait_complete(self) -> bool: ...
    def is_ready(self) -> bool:
        """True when the build finished successfully (archive ready)."""
        ...
    def as_dict(self) -> dict[str, Any]: ...

class Dataset:
    """Dataset metadata record returned by listing / lookup calls."""

    @property
    def dataset_id(self) -> int: ...
    @property
    def dataset_name(self) -> str: ...
    @property
    def dataset_type(self) -> int:
        """1 = Vision, 2 = Text, 3 = Audio, …"""
        ...
    @property
    def dataset_type_interp(self) -> str: ...
    @property
    def dataset_short_info(self) -> str: ...
    @property
    def dataset_status(self) -> int: ...
    @property
    def dataset_status_interp(self) -> str: ...
    @property
    def dataset_tags(self) -> str: ...
    @property
    def publish_type(self) -> int:
        """0 Not Published, 1 Private, 2 Open Source, 3 Public on Demand, 4 Purchase."""
        ...
    @property
    def created_on(self) -> str: ...
    @property
    def modified_on(self) -> str: ...
    @property
    def version_no(self) -> Optional[str]: ...

class DatasetVersionDetails:
    """Version metadata for a specific dataset release."""

    @property
    def id(self) -> int: ...
    @property
    def user_id(self) -> int: ...
    @property
    def dataset_id(self) -> int: ...
    @property
    def version_availability(self) -> int:
        """1 = available."""
        ...
    @property
    def version_no(self) -> str: ...
    @property
    def version_remark(self) -> str: ...
    @property
    def publish_type(self) -> int: ...
    @property
    def created_on(self) -> str: ...
    @property
    def modified_on(self) -> str: ...

class DatasetDownloadDetails:
    """Result returned after a dataset archive has been downloaded and extracted."""

    @property
    def dataset_id(self) -> int: ...
    @property
    def version_no(self) -> str: ...
    @property
    def data_path(self) -> str:
        """Absolute path to the local cache directory containing extracted files."""
        ...
    @property
    def download_status(self) -> bool:
        """True when the archive was successfully downloaded and extracted."""
        ...

class FileInformation:
    """File metadata included in a benchmark result."""

    @property
    def file_name(self) -> str: ...
    @property
    def file_type(self) -> str: ...
    @property
    def file_size(self) -> int: ...
    @property
    def file_hash(self) -> str: ...

class BenchmarkResult:
    """Benchmark result produced by :meth:`Benchmarks.save_benchmark`."""

    @property
    def benchmark_id(self) -> str: ...
    @property
    def model_id(self) -> Optional[int]: ...
    @property
    def model_information(self) -> Optional[list[FileInformation]]: ...
    @property
    def file_information(self) -> Optional[list[FileInformation]]: ...

# ---------------------------------------------------------------------------
# Benchmark verification
# ---------------------------------------------------------------------------

class BenchmarkVerification:
    """Verify benchmark application files against server-side requirements.

    Scans *path* for files listed by the model service and posts their
    fingerprints for server-side validation.
    """

    def __init__(self, server_url: str, benchmark_id: str, path: str) -> None: ...

    def result(self) -> bool:
        """Return ``True`` when the server validates all required files."""
        ...

# ---------------------------------------------------------------------------
# Dataset containers and loaders
# ---------------------------------------------------------------------------

class KappaDataset:
    """Framework-agnostic in-memory dataset.

    Exposes a PyTorch-style ``__len__`` / ``__getitem__`` interface so it can
    be wrapped by any loader (Kappa, PyTorch, TensorFlow, etc.).

    Each sample is a dict::

        {
            "x": <transformed content or raw sample>,
            "y": <target transform output> | missing,
            "entity_id": str,
        }
    """

    def __init__(
        self,
        items: list[DatasetItem],
        transform: Optional[Any] = None,
        target_transform: Optional[Any] = None,
        transform_input_mode: Optional[str] = None,
    ) -> None: ...

    def __len__(self) -> int:
        """Number of samples in the dataset."""
        ...

    def __getitem__(self, idx: int) -> dict[str, Any]:
        """Return a single transformed sample at *idx*."""
        ...

class KappaDataLoader:
    """Map-style, epoch-aware data loader for :class:`KappaDataset`.

    Reshuffles the index permutation at the start of every epoch
    (every ``__iter__`` call). Compatible with PyTorch's ``DataLoader`` API.

    Example::

        loader = client.get_dataset_loader(dataset_name="MyDS", version_no="1.0.0")
        for epoch in range(10):
            for batch in loader:   # list[dict]
                loss = model(batch)
    """

    def __init__(
        self,
        dataset: KappaDataset,
        batch_size: int = 32,
        shuffle: bool = True,
        drop_last: bool = False,
    ) -> None: ...

    def __len__(self) -> int:
        """Number of batches per epoch."""
        ...

    def __iter__(self) -> Iterator[list[dict[str, Any]]]:
        """Reset the epoch (reshuffle + reset position) and return self."""
        ...

    def __next__(self) -> list[dict[str, Any]]:
        """Return the next batch; raises StopIteration when the epoch is done."""
        ...

    def set_dropout_enabled(self, enabled: bool) -> None:
        """Enable entity exclusion filtering on the next epoch."""
        ...

    def set_excluded_entity_ids(self, ids: list[str]) -> None:
        """Replace excluded entity IDs; applied on next ``__iter__`` / epoch reset."""
        ...

    def clear_excluded_entity_ids(self) -> None:
        """Clear exclusions — full dataset on next epoch."""
        ...

    def num_active_samples(self) -> int:
        """Sample count after dropout filter (last epoch reset)."""
        ...

    def excluded_count(self) -> int:
        """Number of samples excluded by the current dropout set."""
        ...

# ---------------------------------------------------------------------------
# Benchmark client
# ---------------------------------------------------------------------------

class Benchmarks:
    """Benchmark workflow handle returned by :meth:`KappaApkClient.load_benchmark`.

    Typical workflow::

        bm = client.load_benchmark("eaa50325-5f3d-4e66-b7b5-b18e5a587563")
        data = bm.dataset()                   # download + load samples
        bm.setup_project()                    # collect source-file fingerprints

        predictions = [{"entity_id": item.entity_id, "predicted": ...} for item in data]
        metrics     = {"accuracy": 0.95, "f1": 0.92}

        bm.save_benchmark(predictions, metrics)
        bm.submit_benchmark()
    """

    @property
    def benchmark_id(self) -> str:
        """The UUID of this benchmark."""
        ...

    def setup_project(self) -> None:
        """Collect random file fingerprints from the project ``src/`` directory.

        Call this before :meth:`save_benchmark` when running the benchmark inside
        an AI/ML application so that source-file metadata is attached to the result.
        """
        ...

    def set_model_id(self, model_id: str) -> None:
        """Override the model ID used when submitting the benchmark result."""
        ...

    def get_model_id(self) -> Optional[str]:
        """Return the active model ID (override if set, otherwise from benchmark details)."""
        ...

    def debug_benchmark_details(self) -> str:
        """Return a human-readable summary of the cached benchmark details."""
        ...

    def dataset(self, dataset_path: Optional[str] = None) -> list[DatasetItem]:
        """Download the benchmark evaluation set and return all samples.

        Follows the same order as the web client: the dataset version package for the
        benchmark's ``datasetId`` / ``datasetVersionNo``
        (``GET /data-micro-services/v2/datasets/versions/{id}/{versionNo}/package``), then
        the benchmark proxy (``GET .../benchmarks/datasets/{benchmark_id}/package``) when
        only ``benchmark.read`` is held. Either path uses the legacy single zip when the
        manifest reports one. Extracted to
        ``~/cache/kappa-framework/benchmarks/{benchmark_id}/``, so a second call with the
        same benchmark skips the download entirely.

        Raises:
            RuntimeError: The version archive is still building (``buildStatus`` is not
                ``ready``).
            PermissionError: A dataset download-approval request is still pending.

        Args:
            dataset_path: Override the cache root directory.  Contents are
                extracted to ``{dataset_path}/{benchmark_id}/``.  Omit to use
                the default cache location.

        Returns:
            A list of :class:`DatasetItem` objects, one per labelled sample.

        Example::

            data = bm.dataset()
            data = bm.dataset("/mnt/scratch")
        """
        ...

    def save_benchmark(
        self,
        predictions: Any,
        metrics: Optional[Any] = None,
        model_path: Optional[str] = None,
    ) -> BenchmarkResult:
        """Build an in-memory benchmark result from predictions and optional metrics.

        Args:
            predictions: A list of dicts, each with ``entity_id``, ``original``,
                and ``predicted`` keys.
            metrics: Optional dict mapping metric names to scalar or aggregate
                values (e.g. ``{"accuracy": 0.95, "f1": 0.92}``).
            model_path: Path to a directory containing model files.  Their
                metadata is included in the result.  Omit if model information
                was already set via :meth:`setup_project`.

        Returns:
            A :class:`BenchmarkResult` that can be submitted via
            :meth:`submit_benchmark`.
        """
        ...

    @property
    def saved_result(self) -> Optional[BenchmarkResult]:
        """The result built by the last :meth:`save_benchmark` call."""
        ...

    def submit_benchmark(
        self,
        strict: bool = True,
        model_version_id: Optional[int] = None,
        complete_inference: bool = True,
        upload_artifacts: bool = False,
        artifact_paths: Optional[list[str]] = None,
        on_progress: Optional[Any] = None,
    ) -> dict[str, Any]:
        """Submit the saved benchmark result to the model service.

        Requires :meth:`save_benchmark` first. When *strict* is True (default),
        validates against the model inference schema before POST.

        With *complete_inference* (default) the saved inference is then linked to the
        benchmark, moving it from *Pending Inference* to *Inference Completed*. The model
        version comes from *model_version_id* or the benchmark's ``mlmodelVersionId``; when
        neither is known the link step is skipped.

        With *upload_artifacts* the model files are uploaded to the new inference —
        *artifact_paths* (files and/or directories) when given, otherwise the ``model_path``
        from :meth:`save_benchmark`. Files past the server's sync cap take a resumable
        multipart session, so multi-GB weights work here. *on_progress* receives
        ``(file_name, bytes_sent, total_bytes, percent)``.

        Example:
            >>> bm.save_benchmark(predictions, metrics, model_path="./model")
            >>> bm.submit_benchmark(upload_artifacts=True)
        """
        ...

# ---------------------------------------------------------------------------
# Dataset utility helper
# ---------------------------------------------------------------------------

class DataLoaderHelper:
    """Static utility helpers for framework adapters."""

    @staticmethod
    def peek_batch(loader: Any) -> Any:
        """Return the first batch from any iterable loader (equivalent to ``next(iter(loader))``)."""
        ...

    @staticmethod
    def infer_tf_output_signature(sample_batch: Any) -> Any:
        """Infer a ``tf.TensorSpec`` output signature from a Kappa sample batch.

        Typical input is one batch produced by :class:`KappaDataLoader`.
        The result can be passed directly to ``tf.data.Dataset.from_generator``.
        """
        ...

# ---------------------------------------------------------------------------
# Dataset request models
# ---------------------------------------------------------------------------

class NewDataset:
    """Request model for creating a new dataset (``POST /datasets/new``)."""

    user_id: int
    dataset_name: str
    dataset_type: int
    dataset_short_info: str
    dataset_tags: str
    dataset_verification_type: int

    def __init__(
        self,
        dataset_name: str,
        dataset_type: int,
        dataset_short_info: str,
        dataset_tags: str,
        user_id: int = 0,
        dataset_verification_type: int = 1,
    ) -> None: ...

    def to_api_json(self) -> str:
        """Serialise to camelCase JSON expected by the dataset service."""
        ...

class UpdateDatasetRequest:
    """Request model for updating dataset metadata (``PUT /datasets/{dataset_id}``)."""

    dataset_name: Optional[str]
    dataset_status: Optional[int]
    remark: Optional[str]
    dataset_verification_type: Optional[int]

    def __init__(
        self,
        dataset_name: Optional[str] = None,
        dataset_status: Optional[int] = None,
        remark: Optional[str] = None,
        dataset_verification_type: Optional[int] = None,
    ) -> None: ...

    def to_api_json(self) -> str: ...

class NewDatasetEntity:
    """Request model for creating a dataset entity (sample)."""

    dataset_id: int
    user_id: int
    ds_entity_name: str
    entity_source: Optional[str]
    collected_on: str
    labeling_algo: str
    ds_entity_info: Any
    location_id: Optional[int]
    files_category: Optional[Any]
    split: Optional[str]

    def __init__(
        self,
        ds_entity_name: str,
        collected_on: str,
        labeling_algo: str,
        ds_entity_info: Any,
        dataset_id: int = 0,
        user_id: int = 0,
        entity_source: Optional[str] = None,
        location_id: Optional[int] = None,
        files_category: Optional[Any] = None,
        split: Optional[str] = None,
    ) -> None: ...

    def to_api_json(self) -> str: ...

class UpdateDatasetEntity:
    """Request model for updating a dataset entity."""

    ds_entity_name: Optional[str]
    ds_entity_status: Optional[int]
    entity_source: Optional[str]
    collected_on: Optional[str]
    labeling_algo: Optional[str]
    ds_entity_info: Optional[Any]
    location_id: Optional[int]
    files_category: Optional[Any]
    version_id: int
    update_latest_entity: bool
    remark: str
    split: Optional[str]

    def __init__(
        self,
        remark: str,
        ds_entity_name: Optional[str] = None,
        ds_entity_status: Optional[int] = None,
        entity_source: Optional[str] = None,
        collected_on: Optional[str] = None,
        labeling_algo: Optional[str] = None,
        ds_entity_info: Optional[Any] = None,
        location_id: Optional[int] = None,
        files_category: Optional[Any] = None,
        version_id: int = 0,
        update_latest_entity: bool = False,
        split: Optional[str] = None,
    ) -> None: ...

    def to_api_json(self) -> str: ...

class DatasetLabel:
    """A single label record returned by ``GET /datasets/labels/{dataset_id}``."""

    @property
    def label_id(self) -> int: ...
    @property
    def dataset_id(self) -> int: ...
    @property
    def label(self) -> str: ...
    @property
    def created_on(self) -> Optional[str]: ...
    @property
    def modified_on(self) -> Optional[str]: ...

class UpdateDatasetLabel:
    """Request body for renaming a dataset label (``PUT /datasets/labels/{dataset_id}``)."""

    label_id: int
    label: str

    def __init__(self, label_id: int, label: str) -> None: ...
    def to_api_json(self) -> str: ...

class DeleteDatasetEntities:
    """Request body for bulk soft-deleting entities (``DELETE /datasets/datasetEntities``)."""

    dataset_entity_ids: list[str]
    remark: str
    version_id: Optional[int]

    def __init__(
        self,
        dataset_entity_ids: list[str],
        remark: str,
        version_id: Optional[int] = None,
    ) -> None: ...

    def to_api_json(self) -> str: ...

class NewDatasetVersion:
    """Request body for creating a new dataset version (``POST /datasets/versions/new/{dataset_id}``)."""

    version_availability: int
    version_type: Optional[str]
    version_remark: Optional[str]

    def __init__(
        self,
        version_availability: int = 1,
        version_type: Optional[str] = None,
        version_remark: Optional[str] = None,
    ) -> None: ...

    def to_api_json(self) -> str: ...

# ---------------------------------------------------------------------------
# User profile types
# ---------------------------------------------------------------------------

class UserTypeDetails:
    user_type_id: int
    user_type: Optional[str]

class OrgDetails:
    org_id: int
    org_name: Optional[str]

class User:
    """Authenticated user profile from ``GET /user-micro-services/v2/users/me``.

    v2 API: ``orgId`` and ``orgDetails`` are optional — both may be ``null`` for
    users without an organization (independent / orgless accounts).
    """

    user_id: int
    user_name: Optional[str]
    first_name: Optional[str]
    middle_name: Optional[str]
    last_name: Optional[str]
    email: Optional[str]
    user_type_id: int
    org_id: Optional[int]
    user_type_details: UserTypeDetails
    org_details: Optional[OrgDetails]
    profile_pic: Optional[str]
    token: Optional[str]
    token_expiry_date: Optional[str]

# ---------------------------------------------------------------------------
# Main client
# ---------------------------------------------------------------------------

class KappaApkClient:
    """HTTP client for authenticating and interacting with the Kappa-framework platform.

    All API calls go through the Traefik API gateway at *base_url*.

    Example::

        with KappaApkClient("http://192.168.0.10:8060", "user@example.com", "secret") as client:
            datasets = client.list_datasets_typed()
            bm = client.load_benchmark("eaa50325-5f3d-4e66-b7b5-b18e5a587563")
            data = bm.dataset()
    """

    def __init__(self, base_url: str, login_id: str, passwd: str) -> None: ...

    # --- context manager ---

    def __enter__(self) -> KappaApkClient:
        """Connect and return self."""
        ...

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> bool:
        """Close session on exit; suppresses no exceptions."""
        ...

    # --- auth ---

    def connect(self) -> dict[str, Any]:
        """Authenticate and return user info + token as a dict."""
        ...

    def close(self) -> None:
        """Invalidate the session token on the server and clear local auth state."""
        ...

    def get_token(self) -> Optional[str]:
        """Return the current bearer token, or None if not authenticated."""
        ...

    def is_authenticated(self) -> bool: ...

    def get_user_profile(self) -> User:
        """Return the authenticated user's profile (``GET /user-micro-services/v2/users/me``)."""
        ...

    def get_my_permissions(
        self,
        dataset_id: Optional[int] = None,
        org_id: Optional[int] = None,
    ) -> dict[str, Any]:
        """Effective permissions (``GET /users/me/permissions``)."""
        ...

    def has_permission(
        self,
        code: str,
        dataset_id: Optional[int] = None,
        org_id: Optional[int] = None,
    ) -> bool:
        """Whether *code* (e.g. ``dataset.write``) is granted for the optional scopes."""
        ...

    def get_system_config(self, tag: str) -> list[dict[str, Any]]:
        """``GET /user-micro-services/v2/system/config/{tag}`` (e.g. ``dataset_tags_1``)."""
        ...

    def list_predefined_ml_tags(self, type_id: int) -> list[str]:
        """Display values from ``dataset_tags_{type_id}`` (dataset/model create catalog)."""
        ...

    def get_base_url(self) -> str: ...
    def set_base_url(self, url: str) -> None: ...

    # --- dataset listing ---

    def list_datasets(
        self,
        page: Optional[int] = None,
        size: Optional[int] = None,
        order_by: Optional[str] = None,
        order_keyword: Optional[str] = None,
    ) -> dict[str, Any]:
        """Return paginated dataset list as a raw JSON dict (default: 200 per page)."""
        ...

    def list_datasets_typed(
        self,
        page: Optional[int] = None,
        size: Optional[int] = None,
        order_by: Optional[str] = None,
        order_keyword: Optional[str] = None,
    ) -> list[Dataset]:
        """Return paginated dataset list as typed :class:`Dataset` objects."""
        ...

    # --- dataset version / archive ---

    def get_dataset_version_details(
        self,
        dataset_id: Optional[int] = None,
        dataset_name: Optional[str] = None,
        version_id: Optional[int] = None,
        version_no: Optional[str] = None,
    ) -> DatasetVersionDetails: ...

    def download_dataset_version_archive(
        self,
        dataset_id: Optional[int] = None,
        dataset_name: Optional[str] = None,
        version_id: Optional[int] = None,
        version_no: Optional[str] = None,
        dataset_path: Optional[str] = None,
    ) -> DatasetDownloadDetails:
        """Download and extract the legacy single-zip version archive to the local cache.

        The archive is cached at ``~/cache/kappa-framework/datasets/{name}_{ver}/``.
        A second call with the same parameters skips the download.

        On Kappa ≥ 2.11 this zip only exists for single-shard versions; use
        :meth:`download_dataset_version_package` for large versions.
        """
        ...

    def download_dataset_version_package(
        self,
        dataset_id: Optional[int] = None,
        dataset_name: Optional[str] = None,
        version_id: Optional[int] = None,
        version_no: Optional[str] = None,
        dataset_path: Optional[str] = None,
    ) -> DatasetDownloadDetails:
        """Download a sharded version package (Kappa ≥ 2.11); falls back to legacy zip.

        Raises if the archive build is not yet ``ready``.
        """
        ...

    def get_dataset_version_package_manifest(
        self,
        dataset_id: int,
        version_no: str,
    ) -> dict[str, Any]:
        """Fetch ``manifest.json`` for a sharded version package."""
        ...

    def load_kappa_dataset(
        self,
        dataset_id: Optional[int] = None,
        dataset_name: Optional[str] = None,
        version_id: Optional[int] = None,
        version_no: Optional[str] = None,
        dataset_path: Optional[str] = None,
        transform: Optional[Any] = None,
        target_transform: Optional[Any] = None,
        transform_input_mode: Optional[str] = None,
        splits: Optional[list[str]] = None,
    ) -> KappaDataset:
        """Download a dataset version and wrap it as a :class:`KappaDataset`.

        *splits*: keep only entities whose ``split`` is in this list
        (missing split counts as ``\"train\"``).
        """
        ...

    def get_dataset_loader(
        self,
        dataset_id: Optional[int] = None,
        dataset_name: Optional[str] = None,
        version_id: Optional[int] = None,
        version_no: Optional[str] = None,
        dataset_path: Optional[str] = None,
        loader_type: Optional[str] = None,
        batch_size: Optional[int] = None,
        shuffle: Optional[bool] = None,
        drop_last: Optional[bool] = None,
        tf_output_signature: Optional[Any] = None,
        transform: Optional[Any] = None,
        target_transform: Optional[Any] = None,
        transform_input_mode: Optional[str] = None,
        splits: Optional[list[str]] = None,
    ) -> Any:
        """Return a data loader for the requested framework.

        *loader_type* selects the backend:

        * ``"kappa"`` (default) — :class:`KappaDataLoader` (Rust, epoch-aware)
        * ``"pytorch"`` — ``torch.utils.data.DataLoader``
        * ``"transformers"`` — HuggingFace Datasets / ``default_data_collator``
        * ``"tensorflow"`` — ``tf.data.Dataset`` (requires *tf_output_signature*)

        *splits*: optional filter by ``entity_info.split`` (see :meth:`load_kappa_dataset`).
        """
        ...

    # --- dataset CRUD ---

    def add_dataset(self, dataset: Any, check_tags: bool = True) -> dict[str, Any]:
        """Create a dataset. With *check_tags*, first ``dataset_tags`` entry must be predefined."""
        ...

    def update_dataset(self, dataset_id: int, update: Any) -> dict[str, Any]:
        """Update dataset metadata (accepts :class:`UpdateDatasetRequest` or a plain dict)."""
        ...

    def add_dataset_entity(
        self,
        dataset_id: int,
        entity: Any,
        file_paths: Optional[list[str]] = None,
        file_category: Optional[str] = None,
        split: Optional[str] = None,
    ) -> dict[str, Any]:
        """Add a labelled entity to a dataset with optional file attachments.

        *file_paths* entries may be local file paths, local directory paths
        (immediate children only), or ``http://`` / ``https://`` URLs.
        *file_category*: ``\"input\"`` (default) or ``\"output\"`` — builds
        ``filesCategory`` for resolved filenames when attaching files.
        *split*: optional value for ``dsEntityInfo.split`` (e.g. train/validation/test).
        """
        ...

    def update_dataset_entity(
        self,
        dataset_id: int,
        entity_id: str,
        update: Any,
        file_paths: Optional[list[str]] = None,
        file_category: Optional[str] = None,
        split: Optional[str] = None,
    ) -> dict[str, Any]: ...

    # --- dataset lookup ---

    def get_dataset_details(
        self,
        dataset_id: Optional[int] = None,
        dataset_name: Optional[str] = None,
    ) -> Dataset:
        """Return full metadata for a dataset by ID or name."""
        ...

    def filter_datasets(
        self,
        search: Optional[str] = None,
        dataset_id: Optional[int] = None,
        dataset_name: Optional[str] = None,
        dataset_type: Optional[int] = None,
        dataset_tags: Optional[str] = None,
        dataset_status: Optional[int] = None,
        publish_type: Optional[int] = None,
        page: Optional[int] = None,
        size: Optional[int] = None,
        order_by: Optional[str] = None,
        order_keyword: Optional[str] = None,
        query_all: Optional[bool] = None,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
        selected_version_id: Optional[int] = None,
        selected_version_no: Optional[str] = None,
    ) -> dict[str, Any]:
        """Filter datasets with rich query params.

        Pages are 1-based; the response is ``{items, total, page, size, pages}``.
        *dataset_tags* is a comma-separated string, e.g. ``"vision,classification"``.
        *publish_type*: 0 Not Published, 1 Private, 2 Open Source, 3 Public on Demand, 4 Purchase.
        *query_all* ``False`` limits results to datasets you own, are assigned to, or that are
        shared with you; the default (``True``) also lists the public catalogue.
        *selected_version_id* / *selected_version_no* add ``selectedVersionNo`` and
        ``selectedVersionBuildStatus`` to every item.
        """
        ...

    def get_dataset_fields(self, dataset_id: int) -> dict[str, Any]:
        """Return the field/schema definition for a dataset."""
        ...

    def delete_dataset(
        self,
        dataset_id: int,
        remark: Optional[str] = None,
    ) -> dict[str, Any]:
        """Soft-delete a dataset (sets ``datasetStatus = 0``).

        Recover with :meth:`recover_datasets`.
        """
        ...

    def recover_datasets(self, dataset_ids: list[int]) -> dict[str, Any]:
        """Recover soft-deleted datasets (``POST .../datasets/recover``)."""
        ...

    def check_dataset_name_availability(self, dataset_name: str) -> dict[str, Any]:
        """Check dataset name uniqueness (``GET .../nameAvailability``)."""
        ...

    # --- label management ---

    def add_dataset_labels(
        self,
        dataset_id: int,
        labels: list[str],
    ) -> dict[str, Any]:
        """Add one or more label strings to a dataset."""
        ...

    def get_dataset_labels(self, dataset_id: int) -> list[DatasetLabel]:
        """List all labels for a dataset as typed :class:`DatasetLabel` objects."""
        ...

    def get_dataset_label_names(self, dataset_id: int) -> list[str]:
        """List label strings for a dataset in API order."""
        ...

    def update_dataset_label(
        self,
        dataset_id: int,
        label_id: int,
        label: str,
    ) -> dict[str, Any]:
        """Rename an existing label by its ID."""
        ...

    # --- entity read operations ---

    def list_dataset_entities(
        self,
        dataset_id: int,
        version_id: Optional[int] = None,
    ) -> dict[str, Any]:
        """List all entities (samples) in a dataset version.

        Deprecated: unpaginated backend route. Prefer :meth:`filter_dataset_entities`.
        """
        ...

    def get_dataset_entity(
        self,
        dataset_id: int,
        entity_id: str,
        version_id: Optional[int] = None,
    ) -> dict[str, Any]:
        """Get a single entity by its string ID."""
        ...

    def filter_dataset_entities(
        self,
        dataset_id: int,
        entity_name: Optional[str] = None,
        entity_status: Optional[int] = None,
        version_id: Optional[int] = None,
        page: Optional[int] = None,
        size: Optional[int] = None,
        order_by: Optional[str] = None,
        order: Optional[str] = None,
        entity_id: Optional[str] = None,
        location_id: Optional[int] = None,
        assignment_filter: Optional[str] = None,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
    ) -> dict[str, Any]:
        """Paginated entity search with optional name / status / version filters.

        Pages are 1-based (default ``page=1``). Sort direction is *order* here, while
        dataset filters use *order_keyword*. *assignment_filter* is ``"assigned"`` or
        ``"not_assigned"`` (default). There is no server-side split filter — read
        ``dsEntityInfo.split`` from each item.
        """
        ...

    def delete_dataset_entities(
        self,
        dataset_entity_ids: list[str],
        remark: str,
        version_id: Optional[int] = None,
    ) -> dict[str, Any]:
        """Bulk soft-delete entities (Kappa ≥ 2.11 returns ``jobId`` — poll mutation job)."""
        ...

    def recover_dataset_entities(
        self,
        dataset_entity_ids: list[str],
        version_id: Optional[int] = None,
    ) -> dict[str, Any]:
        """Recover soft-deleted entities (Kappa ≥ 2.11 returns ``jobId``)."""
        ...

    def upload_dataset_entity_files(
        self,
        dataset_id: int,
        entity_id: str,
        file_paths: list[str],
        file_category: Optional[str] = None,
        check_permission: bool = False,
    ) -> dict[str, Any]:
        """Upload files onto an existing entity (``file_category``: input|output; max 2 GB)."""
        ...

    def delete_dataset_entity_files(
        self,
        entity_file_ids: list[str],
    ) -> dict[str, Any]:
        """Soft-delete entity files (Kappa ≥ 2.11 returns ``jobId``)."""
        ...

    def bulk_upload_dataset_entities(
        self,
        dataset_id: int,
        file_path: str,
        upload_type: str,
        labeling_algo: str,
        source: Optional[str] = None,
        dataset_schema: Optional[Any] = None,
        bulk_split: Optional[str] = None,
        archive_layout: Optional[str] = None,
        strict: bool = True,
        idempotency_key: Optional[str] = None,
        on_upload_progress: Optional[Any] = None,
        check_permission: bool = False,
    ) -> dict[str, Any]:
        """Start async bulk upload.

        * ``upload_type``: ``archive`` | ``csv``
        * CSV max 2 GB; archive ``.zip`` max 50 GB (streamed)
        * ``archive_layout`` required for archive: ``input_output`` | ``classes``
        * ``on_upload_progress(sent, total, percent)`` — HTTP transfer progress
        """
        ...

    def get_bulk_upload_job(self, dataset_id: int, job_id: str) -> BulkUploadJob: ...
    def list_bulk_upload_jobs(self, dataset_id: int) -> dict[str, Any]: ...
    def cancel_bulk_upload_job(self, dataset_id: int, job_id: str) -> dict[str, Any]: ...
    def cancel_stale_bulk_upload_jobs(self, dataset_id: int) -> dict[str, Any]: ...
    def wait_for_bulk_upload_job(
        self,
        dataset_id: int,
        job_id: str,
        poll_interval_secs: Optional[float] = None,
        timeout_secs: Optional[float] = None,
        on_progress: Optional[Any] = None,
    ) -> BulkUploadJob:
        """Poll until bulk job is terminal; *on_progress(job)* each poll."""
        ...

    def retry_bulk_upload_job(
        self,
        dataset_id: int,
        job_id: str,
        sources: Optional[Any] = None,
    ) -> dict[str, Any]:
        """Retry a bulk upload job (optional overrides in *sources*)."""
        ...

    def mark_dataset_entities_labeled(
        self,
        dataset_id: int,
        dataset_entity_ids: Optional[list[str]] = None,
        remark: Optional[str] = None,
        all_eligible: Optional[bool] = None,
    ) -> dict[str, Any]:
        """Enqueue mark-labeled (pass IDs or ``all_eligible=True``). Returns ``jobId``.

        Inline ID lists are capped server-side (default 5000, HTTP 413); use
        ``all_eligible=True`` for whole-dataset runs.
        """
        ...

    def get_mark_labeled_stats(self, dataset_id: int) -> dict[str, Any]: ...
    def get_self_verify_stats(self, dataset_id: int) -> dict[str, Any]: ...
    def bulk_self_verify_dataset_entities(
        self,
        dataset_id: int,
        status: int,
        comment: Optional[str] = None,
        job_corrections_by_entity: Optional[Any] = None,
    ) -> dict[str, Any]:
        """Enqueue self-verify batch (``status`` 1=Pass, 3=Needs Modification)."""
        ...

    def auto_verify_dataset_entities(self, dataset_id: int) -> dict[str, Any]:
        """Enqueue auto-verify for the dataset."""
        ...

    def get_bulk_mutation_job(self, dataset_id: int, job_id: str) -> BulkMutationJob: ...
    def list_bulk_mutation_jobs(self, dataset_id: int) -> dict[str, Any]: ...
    def cancel_bulk_mutation_job(self, dataset_id: int, job_id: str) -> dict[str, Any]: ...
    def wait_for_bulk_mutation_job(
        self,
        dataset_id: int,
        job_id: str,
        poll_interval_secs: Optional[float] = None,
        timeout_secs: Optional[float] = None,
        on_progress: Optional[Any] = None,
    ) -> BulkMutationJob:
        """Poll until mutation job is terminal (default timeout 1 hour).

        *on_progress(job)* is called each poll.
        """
        ...

    def download_dataset_entity_file(
        self,
        dataset_id: int,
        file_id: str,
        dest_path: str,
        as_attachment: bool = False,
    ) -> str:
        """Download an entity file to *dest_path*; returns the path written."""
        ...

    def get_dataset_custom_schema(
        self,
        dataset_id: int,
        schema_kind: Optional[str] = None,
    ) -> dict[str, Any]: ...
    def put_dataset_custom_schema(self, dataset_id: int, schema: Any) -> dict[str, Any]: ...
    def lock_dataset_custom_schema(
        self,
        dataset_id: int,
        schema_kind: Optional[str] = None,
    ) -> dict[str, Any]: ...
    def unlock_dataset_custom_schema(
        self,
        dataset_id: int,
        schema_kind: Optional[str] = None,
    ) -> dict[str, Any]: ...
    def infer_dataset_custom_schema(
        self,
        dataset_id: int,
        csv_content: str,
        sample_rows: Optional[int] = None,
    ) -> dict[str, Any]: ...
    def add_dataset_custom_schema_column(
        self,
        dataset_id: int,
        column: Any,
        schema_kind: Optional[str] = None,
    ) -> dict[str, Any]: ...

    # --- dataset version management ---

    def create_dataset_version(
        self,
        dataset_id: int,
        version: Any,
    ) -> dict[str, Any]:
        """Create a new dataset version; Kappa ≥ 2.11 returns ``jobId`` / ``buildStatus``."""
        ...

    def list_dataset_versions(
        self,
        dataset_id: int,
        version_availability: Optional[int] = None,
    ) -> dict[str, Any]:
        """List all versions for a dataset.

        *version_availability*: 1 = available (omit to list all).
        """
        ...

    def delete_dataset_version(
        self,
        dataset_id: int,
        version_no: str,
    ) -> dict[str, Any]:
        """Delete a specific dataset version by its version number string."""
        ...

    def publish_dataset_version(
        self,
        dataset_id: int,
        version_no: str,
        publish_type: int,
    ) -> dict[str, Any]:
        """Publish a version (requires archive ``buildStatus=ready`` on Kappa ≥ 2.11).

        *publish_type*: 0 Not Published, 1 Private, 2 Open Source, 3 Public on Demand, 4 Purchase.
        """
        ...

    def recover_dataset_version(self, dataset_id: int, version_no: str) -> dict[str, Any]:
        """Recover a soft-deleted dataset version."""
        ...

    def refresh_dataset_version(self, dataset_id: int, version_no: str) -> dict[str, Any]:
        """Patch-release / refresh (enqueues build job on Kappa ≥ 2.11)."""
        ...

    def get_version_build_job(self, job_id: str) -> VersionBuildJob: ...
    def retry_version_build_job(self, job_id: str) -> dict[str, Any]: ...
    def wait_for_version_build_job(
        self,
        job_id: str,
        poll_interval_secs: Optional[float] = None,
        timeout_secs: Optional[float] = None,
        on_progress: Optional[Any] = None,
    ) -> VersionBuildJob:
        """Poll until version build is terminal (default timeout 1 hour)."""
        ...

    # --- benchmarks ---

    def load_benchmark(self, benchmark_id: str) -> Benchmarks:
        """Load a :class:`Benchmarks` handle for the given benchmark ID."""
        ...

    # --- model registry (no card / no publish) ---

    def filter_models(
        self,
        page: Optional[int] = None,
        size: Optional[int] = None,
        search: Optional[str] = None,
    ) -> dict[str, Any]: ...
    def get_model(self, model_id: str) -> dict[str, Any]: ...
    def create_model(self, model: Any, check_tags: bool = True) -> dict[str, Any]:
        """Create a model. With *check_tags*, first ``mlModelTags`` entry must be predefined."""
        ...
    def update_model(self, model_id: str, update: Any) -> dict[str, Any]: ...
    def delete_model(self, model_id: str, remark: Optional[str] = None) -> dict[str, Any]: ...
    def get_model_history(self, model_id: str) -> dict[str, Any]: ...

    def create_model_version(self, model_id: str, version: Any) -> dict[str, Any]: ...
    def list_model_versions(self, model_id: str) -> dict[str, Any]: ...
    def get_model_version(self, model_id: str, version_id: int) -> dict[str, Any]: ...
    def update_model_version(self, model_id: str, version_id: int, update: Any) -> dict[str, Any]: ...
    def delete_model_version(self, model_id: str, version_id: int) -> dict[str, Any]: ...

    def create_model_inference(self, model_id: str, inference: Any) -> dict[str, Any]: ...
    def list_model_inferences(self, model_id: str) -> dict[str, Any]: ...
    def update_model_inference(self, model_id: str, inference_id: int, update: Any) -> dict[str, Any]: ...
    def upload_model_inference_file(
        self,
        model_id: str,
        inference_id: int,
        file_path: str,
        file_category: Optional[int] = None,
        replace: bool = False,
        use_session: Optional[bool] = None,
    ) -> dict[str, Any]:
        """Upload inference artifact (*file_category* 1–5; default 2 Inference).

        Large files use a multipart upload session automatically: pass ``use_session=True``
        to force it, ``False`` to insist on the plain upload (which the server rejects with
        ``413 USE_KAPPA_APK`` past its sync cap). Sessions upsert by file name, so *replace*
        has no effect on them.
        """
        ...
    def write_model_inference(
        self,
        model_id: str,
        predictions: Optional[Any] = None,
        metrics: Optional[Any] = None,
        inference_result: Optional[Any] = None,
        artifacts: Optional[Any] = None,
        benchmark_id: Optional[str] = None,
        file_category: Optional[int] = None,
        validate: bool = True,
        use_session: Optional[bool] = None,
        skip_existing: bool = True,
        on_progress: Optional[Any] = None,
    ) -> dict[str, Any]:
        """Write an inference result and its artifacts in one call, per the model's schema.

        Reads the model's effective inference schema, shapes *predictions* / *metrics* into
        the ``results.predictions[]`` document it requires, validates the payload
        server-side, creates the inference and uploads *artifacts* — a path, a list of
        paths, or directories of weight shards. Files past the sync cap take a resumable
        multipart session; artifacts already attached with the same name and size are
        skipped when *skip_existing*.

        Pass *inference_result* to send a document you built yourself; *predictions* and
        *metrics* are then ignored.

        Prediction dicts accept ``entity_id`` or ``entityId``, and a bare label string for
        ``predicted`` is wrapped into the key the schema requires (e.g. ``class_name``).

        Args:
            artifacts: File path, directory, or list of either.
            benchmark_id: Written into the result document when the run is a benchmark.
            file_category: 1 Training, 2 Inference, 3 Model (default here), 4 Data, 5 Other.
            validate: Validate against the schema before creating the inference.
            use_session: Force (``True``) or forbid (``False``) upload sessions; ``None``
                picks per file size.
            on_progress: Called as ``(file_name, bytes_sent, total_bytes, percent)``.

        Returns:
            ``{"modelId", "inferenceId", "schema", "validation", "artifacts",
            "inferenceResult"}``.

        Raises:
            ValueError: The result does not match the model's inference schema.

        Example:
            >>> written = client.write_model_inference(
            ...     model_id,
            ...     predictions=[{"entityId": "e1", "predicted": {"class_name": "pizza"}}],
            ...     metrics={"accuracy": 0.93},
            ...     artifacts=["./checkpoints", "./config.json"],
            ... )
            >>> written["inferenceId"]
            12
        """
        ...
    def upload_model_artifacts(
        self,
        model_id: str,
        inference_id: int,
        paths: Any,
        file_category: Optional[int] = None,
        use_session: Optional[bool] = None,
        skip_existing: bool = True,
        checksum: Optional[bool] = None,
        on_progress: Optional[Any] = None,
    ) -> list[dict[str, Any]]:
        """Upload a set of artifacts (files, directories, weight shards) to one inference.

        Files go up one at a time — the server admits only a couple of concurrent large
        uploads per model — and each picks its own transport: the plain upload for sidecars,
        a resumable multipart session past the sync cap. With *skip_existing* the package
        manifest is read first, so a re-run after a failure only sends what is missing.

        *on_progress* receives ``(file_name, bytes_sent, total_bytes, percent)``. Each
        returned entry has ``fileName``, ``path``, ``bytes``, ``transport``
        (``"sync"`` / ``"session"`` / ``"skipped"``), ``fileId`` and the raw ``response``.
        """
        ...
    def upload_model_artifact_session(
        self,
        model_id: str,
        inference_id: int,
        file_path: str,
        file_category: Optional[int] = None,
        on_progress: Optional[Any] = None,
        checksum: bool = False,
        resume: bool = True,
        max_retries: int = 5,
        wait_for_slot: bool = True,
    ) -> dict[str, Any]:
        """Upload a large artifact through a multipart session.

        *on_progress* receives ``(bytes_sent, total_bytes, percent)`` after each part.
        *file_category* defaults to 3 (Model) on this route, unlike the plain upload's 2.

        Each part is retried up to *max_retries* times against a freshly presigned URL, and
        the upload id plus part ETags are checkpointed under the user cache dir, so an
        interrupted run resumes where it stopped (*resume=False* always starts over). With
        *wait_for_slot* a ``429 MODEL_ARTIFACT_UPLOAD_ADMISSION_LIMIT`` is waited out instead
        of raised. *checksum* computes the file's SHA-256 and records it on the artifact.
        """
        ...
    def get_model_artifact_upload_session(
        self,
        model_id: str,
        inference_id: int,
        upload_id: str,
    ) -> dict[str, Any]: ...
    def abort_model_artifact_upload_session(
        self,
        model_id: str,
        inference_id: int,
        upload_id: str,
    ) -> dict[str, Any]:
        """Abort a pending upload session (frees an admission slot)."""
        ...
    def get_model_inference_artifacts_package(
        self,
        model_id: str,
        inference_id: int,
    ) -> dict[str, Any]:
        """Artifact manifest: ``files[]`` with sizes, categories and optional download URLs."""
        ...
    def get_model_version_artifacts_package(
        self,
        model_id: str,
        version_id: int,
    ) -> dict[str, Any]: ...
    def download_model_inference_artifact_file(
        self,
        model_id: str,
        inference_id: int,
        file_id: str,
        dest_path: str,
        redirect: bool = False,
    ) -> str:
        """Download one artifact by file ID.

        *redirect* follows a presigned object-storage URL instead of streaming through the
        gateway; it only works where that storage is reachable.
        """
        ...
    def download_model_inference_artifacts_package(
        self,
        model_id: str,
        inference_id: int,
        dest_dir: str,
        redirect: bool = False,
    ) -> list[str]:
        """Download every artifact of an inference, file by file.

        Prefer this for big packages: the single zip is refused with
        ``409 PACKAGE_TOO_LARGE_FOR_ZIP`` past the server's zip ceiling. Falls back to the
        zip on backends without package routes. Returns the paths written.
        """
        ...
    def download_model_inference_artifacts(
        self,
        model_id: str,
        inference_id: int,
        dest_path: str,
    ) -> str:
        """Download all inference artifacts as one zip (small packages only)."""
        ...
    def download_model_version_artifacts(
        self,
        model_id: str,
        version_id: int,
        dest_path: str,
    ) -> str:
        """Download a model version's artifacts as one zip (small packages only)."""
        ...
    def get_model_version_inference(self, model_id: str, version_id: int) -> dict[str, Any]: ...

    def get_model_inference_schema(self, model_id: str) -> dict[str, Any]: ...
    def update_model_inference_schema(self, model_id: str, schema: Any) -> dict[str, Any]: ...
    def delete_model_inference_schema(self, model_id: str) -> dict[str, Any]: ...
    def validate_inference_result(self, model_id: str, inference_result: Any) -> dict[str, Any]: ...
    def list_inference_metrics(self) -> dict[str, Any]: ...
    def list_inference_schema_types(self) -> dict[str, Any]: ...
    def get_model_inference_schema_history(
        self,
        model_id: str,
        limit: Optional[int] = None,
    ) -> dict[str, Any]: ...
    def get_inference_schema_type(self, model_type: int) -> dict[str, Any]: ...

    def list_model_pipelines(self, model_id: str) -> dict[str, Any]: ...
    def create_model_pipeline(self, model_id: str, version_id: int, pipeline: Any) -> dict[str, Any]: ...
    def get_model_pipeline(self, model_id: str, version_id: int) -> dict[str, Any]: ...
    def update_model_pipeline(self, model_id: str, version_id: int, pipeline: Any) -> dict[str, Any]: ...
    def delete_model_pipeline(self, model_id: str, version_id: int) -> dict[str, Any]: ...
    def validate_model_pipeline(self, model_id: str, version_id: int) -> dict[str, Any]: ...

    def list_benchmarks(self) -> dict[str, Any]:
        """First page of benchmarks, unfiltered. See :meth:`filter_benchmarks`."""
        ...
    def filter_benchmarks(
        self,
        benchmark_id: Optional[str] = None,
        model_id: Optional[str] = None,
        model_type: Optional[int] = None,
        dataset_id: Optional[int] = None,
        dataset_version_id: Optional[int] = None,
        model_version_id: Optional[int] = None,
        benchmark_status: Optional[int] = None,
        user_id: Optional[int] = None,
        report_id: Optional[int] = None,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
        order_by: Optional[str] = None,
        order: Optional[str] = None,
        page: Optional[int] = None,
        size: Optional[int] = None,
    ) -> dict[str, Any]:
        """Paginated benchmark search. Sort direction is *order* (``"ASC"`` / ``"DESC"``)."""
        ...
    def get_benchmark(self, benchmark_id: str) -> dict[str, Any]:
        """Benchmark detail dict (``datasetId``, ``datasetVersionNo``, ``benchmarkStatus``, …)."""
        ...
    def create_benchmark(self, benchmark: Any) -> dict[str, Any]: ...
    def update_benchmark(self, benchmark_id: str, update: Any) -> dict[str, Any]: ...
    def delete_benchmark(self, benchmark_id: str) -> dict[str, Any]: ...
    def complete_benchmark_inference(self, benchmark_id: str, model_version_id: int) -> dict[str, Any]:
        """Attach a model version's inference results to the benchmark (status 4 → 5)."""
        ...
    def get_benchmark_flow_schema(self) -> dict[str, Any]: ...

    def list_benchmark_remarks(self, benchmark_id: str) -> list[dict[str, Any]]: ...
    def add_benchmark_remark(self, benchmark_id: str, message: str) -> dict[str, Any]:
        """Post a remark (1–4000 characters). Rejected once the benchmark is closed."""
        ...

    def respond_to_benchmark_expert_request(self, benchmark_id: str, accept: bool) -> dict[str, Any]:
        """Accept or reject an expert request as the assigned expert."""
        ...
    def assign_benchmark_expert(self, benchmark_id: str, expert_id: int) -> dict[str, Any]:
        """Assign an expert (requires ``benchmark.manage``)."""
        ...
    def propose_benchmark_dataset(
        self,
        benchmark_id: str,
        dataset_id: int,
        dataset_version_id: int,
    ) -> dict[str, Any]: ...
    def confirm_benchmark_dataset(self, benchmark_id: str) -> dict[str, Any]: ...
    def reject_benchmark_dataset(self, benchmark_id: str) -> dict[str, Any]: ...
    def get_benchmark_dataset_attachments(
        self,
        dataset_id: int,
        dataset_version_id: int,
    ) -> dict[str, Any]: ...

    def get_benchmark_review(self, benchmark_id: str, expert_id: int) -> dict[str, Any]:
        """Inference results plus saved expert reviews. First call moves status 5 → 6."""
        ...
    def save_benchmark_review(
        self,
        benchmark_id: str,
        expert_id: int,
        review: Any,
    ) -> dict[str, Any]:
        """Save review progress (``{"reviews": {entityUuid: {...}}, "finalScore": ...}``)."""
        ...
    def finalize_benchmark_review(self, benchmark_id: str, expert_id: int) -> dict[str, Any]:
        """Finalize the review and schedule report generation. Save the review first."""
        ...
    def regenerate_benchmark_report(self, benchmark_id: str) -> dict[str, Any]: ...
    def get_benchmark_report(self, benchmark_id: str) -> dict[str, Any]: ...
    def download_benchmark_report(
        self,
        benchmark_id: str,
        dest_path: str,
        lang: Optional[str] = None,
    ) -> str:
        """Write the report PDF. *lang* is ``"en"`` (default) or ``"ru"``."""
        ...

    def get_benchmark_dataset_package_manifest(self, benchmark_id: str) -> dict[str, Any]: ...
    def download_benchmark_dataset_package(
        self,
        benchmark_id: str,
        dataset_id: Optional[int] = None,
        version_no: Optional[str] = None,
        dataset_path: Optional[str] = None,
    ) -> str:
        """Download the benchmark evaluation set the way the web client does.

        Tries the dataset version package for the benchmark's ``datasetId`` /
        ``datasetVersionNo`` first, then the benchmark proxy when only ``benchmark.read``
        is held; each path uses the legacy single zip when the manifest says so. Both IDs
        are read from the benchmark detail when omitted. Returns the extraction directory.
        """
        ...

    # --- raw HTTP ---

    def make_request(
        self,
        method: str,
        endpoint: str,
        data: Optional[str] = None,
        token: Optional[str] = None,
    ) -> dict[str, Any]:
        """Make a raw HTTP request against the gateway and return the JSON response."""
        ...
