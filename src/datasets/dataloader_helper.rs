// Copyright 2026 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Utility helpers for framework adapters.
///
/// This class intentionally keeps framework-heavy logic outside the Rust
/// dataset core so we do not duplicate PyTorch / TensorFlow internals.
#[pyclass]
pub struct DataLoaderHelper;

#[pymethods]
impl DataLoaderHelper {
    /// Return one batch from any iterable data loader.
    ///
    /// Equivalent to Python: `next(iter(loader))`.
    #[staticmethod]
    pub fn peek_batch(loader: Py<PyAny>, py: Python<'_>) -> PyResult<PyObject> {
        let builtins = py.import("builtins")?;
        let iter_fn = builtins.getattr("iter")?;
        let next_fn = builtins.getattr("next")?;
        let it = iter_fn.call1((loader.bind(py),))?;
        Ok(next_fn.call1((it,))?.into())
    }

    /// Infer a TensorFlow `output_signature` from a Kappa batch-like object.
    ///
    /// Typical input is one batch produced by `KappaDataLoader` (list[dict]).
    /// The helper returns a nested `tf.TensorSpec` structure with a flexible
    /// batch dimension (`None`) so it can be used with:
    ///
    /// `tf.data.Dataset.from_generator(..., output_signature=signature)`
    #[staticmethod]
    pub fn infer_tf_output_signature(sample_batch: Py<PyAny>, py: Python<'_>) -> PyResult<PyObject> {
        let tf = py.import("tensorflow").map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "TensorFlow is required for infer_tf_output_signature but could not be imported",
            )
        })?;

        let locals = PyDict::new(py);
        locals.set_item("sample_batch", sample_batch.bind(py))?;
        locals.set_item("tf", tf)?;

        py.run(
            pyo3::ffi::c_str!(
                r#"
import numpy as _np

def _kappa_spec_from_obj(obj):
    # dict: recurse by key
    if isinstance(obj, dict):
        return {k: _kappa_spec_from_obj(v) for k, v in obj.items()}

    # list/tuple: treat as batch-like and build None-leading TensorSpec
    if isinstance(obj, (list, tuple)):
        if len(obj) == 0:
            return tf.TensorSpec(shape=(None,), dtype=tf.float32)

        # Common Kappa case: list[dict] from a batch
        if isinstance(obj[0], dict):
            keys = list(obj[0].keys())
            out = {}
            for k in keys:
                vals = [item.get(k) for item in obj]
                out[k] = _kappa_spec_from_obj(vals)
            return out

        # If element itself has shape, infer element and prepend batch dim
        elem = obj[0]
        try:
            # torch tensor path
            import torch as _torch
            if isinstance(elem, _torch.Tensor):
                arr = elem.detach().cpu().numpy()
                return tf.TensorSpec(shape=(None, *arr.shape), dtype=tf.as_dtype(arr.dtype))
        except Exception:
            pass

        try:
            if hasattr(elem, "numpy"):
                arr = elem.numpy()
                return tf.TensorSpec(shape=(None, *arr.shape), dtype=tf.as_dtype(arr.dtype))
        except Exception:
            pass

        # scalar / python-native list
        if isinstance(elem, str):
            return tf.TensorSpec(shape=(None,), dtype=tf.string)
        if isinstance(elem, bool):
            return tf.TensorSpec(shape=(None,), dtype=tf.bool)
        if isinstance(elem, int):
            return tf.TensorSpec(shape=(None,), dtype=tf.int64)
        if isinstance(elem, float):
            return tf.TensorSpec(shape=(None,), dtype=tf.float32)

        arr = _np.array(obj)
        return tf.TensorSpec(shape=(None, *arr.shape[1:]), dtype=tf.as_dtype(arr.dtype))

    # leaf values
    try:
        import torch as _torch
        if isinstance(obj, _torch.Tensor):
            arr = obj.detach().cpu().numpy()
            return tf.TensorSpec(shape=arr.shape, dtype=tf.as_dtype(arr.dtype))
    except Exception:
        pass

    try:
        if hasattr(obj, "numpy"):
            arr = obj.numpy()
            return tf.TensorSpec(shape=arr.shape, dtype=tf.as_dtype(arr.dtype))
    except Exception:
        pass

    if isinstance(obj, str):
        return tf.TensorSpec(shape=(), dtype=tf.string)
    if isinstance(obj, bool):
        return tf.TensorSpec(shape=(), dtype=tf.bool)
    if isinstance(obj, int):
        return tf.TensorSpec(shape=(), dtype=tf.int64)
    if isinstance(obj, float):
        return tf.TensorSpec(shape=(), dtype=tf.float32)

    arr = _np.array(obj)
    return tf.TensorSpec(shape=arr.shape, dtype=tf.as_dtype(arr.dtype))

signature = _kappa_spec_from_obj(sample_batch)
"#
            ),
            None,
            Some(&locals),
        )?;

        let signature = locals
            .get_item("signature")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to infer signature"))?;
        Ok(signature.into())
    }

    /// Build a TensorFlow loader from a Kappa client in one step.
    ///
    /// `loader_kwargs` should contain the same keys you pass to
    /// `client.get_dataset_loader(...)` (dataset/version ids, batch_size, shuffle, etc.).
    /// This helper will:
    /// 1) create a temporary Kappa loader,
    /// 2) peek one batch,
    /// 3) infer `tf_output_signature`,
    /// 4) create and return a TensorFlow loader.
    #[staticmethod]
    #[pyo3(signature = (client, loader_kwargs=None))]
    pub fn make_tf_loader(
        client: Py<PyAny>,
        loader_kwargs: Option<Py<PyDict>>,
        py: Python<'_>,
    ) -> PyResult<PyObject> {
        let kwargs = PyDict::new(py);
        if let Some(src) = loader_kwargs {
            for (k, v) in src.bind(py).iter() {
                kwargs.set_item(k, v)?;
            }
        }

        // Build Kappa loader first for peeking and signature inference.
        kwargs.set_item("loader_type", "kappa")?;
        let kappa_loader = client
            .bind(py)
            .call_method("get_dataset_loader", (), Some(&kwargs))?;

        let sample_batch = Self::peek_batch(kappa_loader.into(), py)?;
        let tf_signature = Self::infer_tf_output_signature(sample_batch, py)?;

        // Build TensorFlow loader with inferred signature.
        kwargs.set_item("loader_type", "tensorflow")?;
        kwargs.set_item("tf_output_signature", tf_signature)?;
        let tf_loader = client
            .bind(py)
            .call_method("get_dataset_loader", (), Some(&kwargs))?;

        Ok(tf_loader.into())
    }
}

