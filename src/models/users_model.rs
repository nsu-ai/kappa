// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

// Response structures for user-micro-services the API
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserTypeDetails {
    pub user_type_id: i32,
    #[serde(default)]
    pub user_type: Option<String>,
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgDetails {
    pub org_id: i32,
    #[serde(default)]
    pub org_name: Option<String>,
}

/// User entity exposed to Python.
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub user_id: i32,
    #[serde(default)]
    pub user_name: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub middle_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    pub user_type_id: i32,
    pub org_id: i32,
    pub user_type_details: UserTypeDetails,
    pub org_details: OrgDetails,
    #[serde(default)]
    pub profile_pic: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub token_expiry_date: Option<String>,
}