// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub login_id: String,
    pub passwd: String,
}

impl Drop for LoginRequest {
    fn drop(&mut self) {
        self.passwd.zeroize();
    }
}