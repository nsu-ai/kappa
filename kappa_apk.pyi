# Type stubs for kappa_apk — auto-maintained alongside src/
# Covers all Python-accessible classes, methods, and properties.

from __future__ import annotations
from typing import Any, Iterator, Optional

def version() -> str:
    """Return the library version string (kf-sdk / kappa_apk)."""
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
        """Download the benchmark dataset archive and return all samples.

        The archive is fetched from
        ``GET /model-micro-services/v2/benchmarks/datasets/download/{benchmark_id}``
        and extracted to the local cache under
        ``~/cache/kappa-framework/benchmarks/{benchmark_id}/``.
        A second call with the same benchmark skips the download entirely.

        Args:
            dataset_path: Override the cache root directory.  The archive is
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

    def submit_benchmark(self, strict: bool = True) -> dict[str, Any]:
        """Submit the saved benchmark result to the model service.

        Requires :meth:`save_benchmark` first. When *strict* is True (default),
        validates against the model inference schema before POST.
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
        """Download and extract a dataset version archive to the local cache.

        The archive is cached at ``~/cache/kappa-framework/datasets/{name}_{ver}/``.
        A second call with the same parameters skips the download.
        """
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

    def add_dataset(self, dataset: Any) -> dict[str, Any]:
        """Create a new dataset (accepts :class:`NewDataset` or a plain dict)."""
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
    ) -> dict[str, Any]:
        """Filter datasets with rich query params.

        *dataset_tags* is a comma-separated string, e.g. ``"vision,classification"``.
        *publish_type*: 0 Not Published, 1 Private, 2 Open Source, 3 Public on Demand, 4 Purchase.
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
    ) -> dict[str, Any]:
        """Paginated entity search with optional name / status / version filters."""
        ...

    def delete_dataset_entities(
        self,
        dataset_entity_ids: list[str],
        remark: str,
        version_id: Optional[int] = None,
    ) -> dict[str, Any]:
        """Bulk soft-delete entities by their string IDs."""
        ...

    def recover_dataset_entities(
        self,
        dataset_entity_ids: list[str],
        version_id: Optional[int] = None,
    ) -> dict[str, Any]:
        """Recover soft-deleted entities (``POST .../datasetEntities/recover``)."""
        ...

    def upload_dataset_entity_files(
        self,
        dataset_id: int,
        entity_id: str,
        file_paths: list[str],
        file_category: Optional[str] = None,
    ) -> dict[str, Any]:
        """Upload files onto an existing entity (``file_category``: input|output)."""
        ...

    def delete_dataset_entity_files(
        self,
        entity_file_ids: list[str],
    ) -> dict[str, Any]:
        """Soft-delete entity files by file ID list."""
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
        strict: bool = True,
        idempotency_key: Optional[str] = None,
    ) -> dict[str, Any]:
        """Start async bulk upload (``upload_type``: archive|csv)."""
        ...

    def get_bulk_upload_job(self, dataset_id: int, job_id: str) -> dict[str, Any]: ...
    def list_bulk_upload_jobs(self, dataset_id: int) -> dict[str, Any]: ...
    def cancel_bulk_upload_job(self, dataset_id: int, job_id: str) -> dict[str, Any]: ...
    def cancel_stale_bulk_upload_jobs(self, dataset_id: int) -> dict[str, Any]: ...
    def wait_for_bulk_upload_job(
        self,
        dataset_id: int,
        job_id: str,
        poll_interval_secs: Optional[float] = None,
        timeout_secs: Optional[float] = None,
    ) -> dict[str, Any]:
        """Poll until bulk job reaches a terminal status or times out."""
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
        dataset_entity_ids: list[str],
        remark: Optional[str] = None,
    ) -> dict[str, Any]:
        """Mark default-algorithm entities as labeled."""
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
        """Create a new dataset version (accepts :class:`NewDatasetVersion` or a plain dict)."""
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
        """Publish a dataset version.

        *publish_type*: 0 Not Published, 1 Private, 2 Open Source, 3 Public on Demand, 4 Purchase.
        """
        ...

    def recover_dataset_version(self, dataset_id: int, version_no: str) -> dict[str, Any]:
        """Recover a soft-deleted dataset version."""
        ...

    def refresh_dataset_version(self, dataset_id: int, version_no: str) -> dict[str, Any]:
        """Rebuild the version archive after entity changes."""
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
    def create_model(self, model: Any) -> dict[str, Any]: ...
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
    ) -> dict[str, Any]:
        """Upload inference artifact (*file_category* 1–5; default 2 Inference)."""
        ...
    def download_model_inference_artifacts(
        self,
        model_id: str,
        inference_id: int,
        dest_path: str,
    ) -> str: ...
    def download_model_version_artifacts(
        self,
        model_id: str,
        version_id: int,
        dest_path: str,
    ) -> str: ...
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

    def list_benchmarks(self) -> dict[str, Any]: ...
    def create_benchmark(self, benchmark: Any) -> dict[str, Any]: ...
    def update_benchmark(self, benchmark_id: str, update: Any) -> dict[str, Any]: ...
    def delete_benchmark(self, benchmark_id: str) -> dict[str, Any]: ...
    def complete_benchmark_inference(self, benchmark_id: str, model_version_id: int) -> dict[str, Any]: ...

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
