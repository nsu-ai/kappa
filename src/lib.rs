// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;

// Core modules
mod client;
mod traits;
mod models;
mod users;
mod benchmarks;
mod utils;
pub mod transforms;

// Public API modules
pub mod datasets;

// Re-export main types for external use
use client::KappaApkClient;
use datasets::dataloader_helper::DataLoaderHelper;
use datasets::datasets::KappaDataset;
use datasets::kappa_dataloader::KappaDataLoader;
use models::datasets_model::{
    DatasetLabel, DeleteDatasetEntities, NewDataset, NewDatasetEntity, NewDatasetVersion,
    UpdateDatasetEntity, UpdateDatasetLabel, UpdateDatasetRequest,
};
use models::datasets_model::{
    Dataset, DatasetDownloadDetails, DatasetItem, DatasetVersionDetails, ItemFile,
};
use models::users_model::{OrgDetails, User, UserTypeDetails};
pub use traits::*;
use crate::benchmarks::verifications::BenchmarkVerification;

/// Returns the version of the kf-sdk library.
#[pyfunction]
fn version() -> PyResult<String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn kappa_apk(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_class::<KappaApkClient>()?;
    m.add_class::<User>()?;
    m.add_class::<UserTypeDetails>()?;
    m.add_class::<OrgDetails>()?;
    m.add_class::<BenchmarkVerification>()?;
    m.add_class::<KappaDataset>()?;
    m.add_class::<KappaDataLoader>()?;
    m.add_class::<DataLoaderHelper>()?;
    m.add_class::<Dataset>()?;
    m.add_class::<DatasetVersionDetails>()?;
    m.add_class::<DatasetDownloadDetails>()?;
    m.add_class::<DatasetItem>()?;
    m.add_class::<ItemFile>()?;
    m.add_class::<DatasetLabel>()?;
    m.add_class::<NewDataset>()?;
    m.add_class::<UpdateDatasetRequest>()?;
    m.add_class::<NewDatasetEntity>()?;
    m.add_class::<UpdateDatasetEntity>()?;
    m.add_class::<UpdateDatasetLabel>()?;
    m.add_class::<DeleteDatasetEntities>()?;
    m.add_class::<NewDatasetVersion>()?;
    transforms::vision::register(m)?;
    transforms::text::register(m)?;
    transforms::audio::register(m)?;
    Ok(())
}
