// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! Map-style batch loader aligned with common [`torch.utils.data.DataLoader`](https://github.com/pytorch/pytorch/blob/main/torch/utils/data/dataloader.py) usage:
//! each `for batch in loader:` calls `__iter__` at epoch start, which resets position and
//! reshuffles when `shuffle=True`. Each batch sample includes `entity_id` from the underlying dataset.
//!
//! Optional entity exclusion: call `set_excluded_entity_ids` before the next epoch to filter samples.

use super::datasets::KappaDataset;
use pyo3::prelude::*;
use pyo3::types::PyList;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;

/// Apply entity-ID dropout filtering over dataset indices.
///
/// Returns the filtered index list when enough samples remain; otherwise returns the
/// full index range unchanged.
pub(crate) fn filter_indices_for_dropout(
    len: usize,
    entity_id_at: impl Fn(usize) -> String,
    excluded: &HashSet<String>,
    min_remaining: usize,
) -> Vec<usize> {
    let indices: Vec<usize> = (0..len).collect();
    if excluded.is_empty() {
        return indices;
    }

    let filtered: Vec<usize> = indices
        .iter()
        .copied()
        .filter(|&i| !excluded.contains(&entity_id_at(i)))
        .collect();

    if filtered.len() >= min_remaining {
        filtered
    } else {
        indices
    }
}

/// Iterable batch loader for [`KappaDataset`](crate::datasets::datasets::KappaDataset).
///
/// Epoch semantics match the usual PyTorch pattern: every `iter(loader)` (including each
/// `for batch in loader:` loop) starts a fresh epoch with `position` reset and, if
/// `shuffle` is true, a new random permutation. Exhaustion is signaled with
/// `StopIteration` only; there is no hidden reshuffle at the end of `__next__`.
///
/// Each sample in a batch is the same dict as `KappaDataset.__getitem__` (including
/// `entity_id`), so feedback tracing stays correct regardless of shuffle order.
#[pyclass]
pub struct KappaDataLoader {
    dataset: Py<KappaDataset>,
    batch_size: usize,
    shuffle: bool,
    drop_last: bool,
    indices: Vec<usize>,
    position: usize,
    dropout_enabled: bool,
    excluded_entity_ids: HashSet<String>,
    min_remaining_entities: usize,
    active_sample_count: usize,
}

impl KappaDataLoader {
    pub(crate) fn new_internal(
        dataset: Py<KappaDataset>,
        batch_size: usize,
        shuffle: bool,
        drop_last: bool,
        py: Python<'_>,
    ) -> PyResult<Self> {
        let mut loader = KappaDataLoader {
            dataset,
            batch_size: if batch_size == 0 { 1 } else { batch_size },
            shuffle,
            drop_last,
            indices: Vec::new(),
            position: 0,
            dropout_enabled: false,
            excluded_entity_ids: HashSet::new(),
            min_remaining_entities: 1,
            active_sample_count: 0,
        };
        loader.reset_epoch(py)?;
        Ok(loader)
    }

    fn build_filtered_indices(&self, py: Python<'_>) -> PyResult<Vec<usize>> {
        let dataset_ref = self.dataset.borrow(py);
        let len = dataset_ref.items.len();

        if !self.dropout_enabled || self.excluded_entity_ids.is_empty() {
            return Ok((0..len).collect());
        }

        let strict_filtered_len = (0..len)
            .filter(|&i| !self.excluded_entity_ids.contains(&dataset_ref.items[i].entity_id))
            .count();
        let indices = filter_indices_for_dropout(
            len,
            |i| dataset_ref.items[i].entity_id.clone(),
            &self.excluded_entity_ids,
            self.min_remaining_entities,
        );

        if strict_filtered_len < self.min_remaining_entities && strict_filtered_len < len {
            let warnings = py.import("warnings")?;
            warnings.call_method1(
                "warn",
                (format!(
                    "KappaDataLoader: dropout would leave {} samples (< min_remaining_entities={}); keeping full dataset.",
                    strict_filtered_len,
                    self.min_remaining_entities
                ),),
            )?;
        }

        Ok(indices)
    }

    fn reset_epoch(&mut self, py: Python<'_>) -> PyResult<()> {
        self.indices = self.build_filtered_indices(py)?;
        self.active_sample_count = self.indices.len();
        if self.shuffle && !self.indices.is_empty() {
            let mut rng = thread_rng();
            self.indices.shuffle(&mut rng);
        }
        self.position = 0;
        Ok(())
    }

    fn batch_count_for(&self, sample_count: usize) -> usize {
        let bs = self.batch_size.max(1);
        if self.drop_last {
            sample_count / bs
        } else {
            sample_count.div_ceil(bs)
        }
    }
}

#[pymethods]
impl KappaDataLoader {
    /// Create a new `KappaDataLoader` over a `KappaDataset`.
    ///
    /// Defaults: `batch_size=32`, `shuffle=True`, `drop_last=False` (incomplete last batch
    /// is kept), matching common PyTorch defaults except `drop_last` default.
    #[new]
    #[pyo3(signature = (dataset, batch_size=32, shuffle=true, drop_last=false))]
    pub fn new(
        dataset: Py<KappaDataset>,
        batch_size: usize,
        shuffle: bool,
        drop_last: bool,
        py: Python<'_>,
    ) -> PyResult<Self> {
        KappaDataLoader::new_internal(dataset, batch_size, shuffle, drop_last, py)
    }

    /// Enable or disable bad-entity exclusion on the next epoch.
    pub fn set_dropout_enabled(&mut self, enabled: bool) {
        self.dropout_enabled = enabled;
    }

    /// Replace the set of entity IDs excluded on the next `__iter__` / epoch reset.
    pub fn set_excluded_entity_ids(&mut self, ids: Vec<String>) {
        self.excluded_entity_ids = ids.into_iter().collect();
    }

    /// Clear exclusions — full dataset on next epoch.
    pub fn clear_excluded_entity_ids(&mut self) {
        self.excluded_entity_ids.clear();
    }

    /// Samples available after the last epoch reset (after dropout filter).
    pub fn num_active_samples(&self) -> usize {
        self.active_sample_count
    }

    /// Number of entity IDs currently excluded.
    pub fn excluded_count(&self, py: Python<'_>) -> PyResult<usize> {
        let total = self.dataset.borrow(py).items.len();
        Ok(total.saturating_sub(self.active_sample_count))
    }

    /// Start (or restart) an epoch: reset read position and reshuffle if `shuffle` is enabled.
    fn __iter__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRefMut<'_, Self>> {
        Python::with_gil(|py| slf.reset_epoch(py))?;
        Ok(slf)
    }

    /// Number of batches per epoch (uses filtered sample count when dropout is active).
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        let n = self.build_filtered_indices(py)?.len();
        Ok(self.batch_count_for(n))
    }

    /// Next batch as a `list` of transformed samples, or end iteration.
    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        let n = self.indices.len();
        if self.position >= n {
            return Ok(None);
        }

        let remaining = n - self.position;
        if self.drop_last && remaining < self.batch_size {
            self.position = n;
            return Ok(None);
        }

        let end = (self.position + self.batch_size).min(n);
        let batch_indices = &self.indices[self.position..end];
        self.position = end;

        let dataset_ref = self.dataset.borrow(py);
        let mut batch: Vec<PyObject> = Vec::with_capacity(batch_indices.len());
        for &idx in batch_indices {
            let sample = dataset_ref.build_transformed_sample(idx, py)?;
            batch.push(sample);
        }

        let py_list = PyList::new(py, batch)?;
        Ok(Some(py_list.into()))
    }
}

#[cfg(test)]
mod dropout_tests {
    use super::*;

    #[test]
    fn dropout_excludes_matching_entities() {
        let entity_ids = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let excluded: HashSet<String> = ["b".to_string()].into_iter().collect();
        let indices = filter_indices_for_dropout(
            entity_ids.len(),
            |i| entity_ids[i].clone(),
            &excluded,
            1,
        );
        assert_eq!(indices, vec![0, 2]);
    }

    #[test]
    fn dropout_keeps_full_dataset_when_too_few_remain() {
        let entity_ids = vec!["a".to_string(), "b".to_string()];
        let excluded: HashSet<String> = ["a".to_string(), "b".to_string()].into_iter().collect();
        let indices = filter_indices_for_dropout(
            entity_ids.len(),
            |i| entity_ids[i].clone(),
            &excluded,
            1,
        );
        assert_eq!(indices, vec![0, 1]);
    }

    #[test]
    fn dropout_noop_when_excluded_empty() {
        let indices = filter_indices_for_dropout(4, |_| "x".to_string(), &HashSet::new(), 1);
        assert_eq!(indices, vec![0, 1, 2, 3]);
    }
}
