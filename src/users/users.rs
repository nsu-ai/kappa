// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use crate::traits::ApiClient;
use crate::models::users_model::User;
use crate::utils::python_json::pyobject_to_rust_value;

/// Default implementation of Users trait
#[allow(dead_code)]
pub struct Users;

#[allow(dead_code)]
impl Users {
    /// Get current user profile
    /// 
    /// Returns a Python dict mirroring server JSON
    ///
    /// # Python Example
    /// ```python
    /// user = client.get_user_profile()
    /// print(user.user_id, user.user_name)
    /// ```
    pub fn get_user_profile<T: ApiClient>(client: &T) -> PyResult<User> {
        let token = client.require_token()?;
        let endpoint = "/user-micro-services/v2/users/me".to_string();
        let json_obj = client.make_request("GET".to_string(), endpoint, None, Some(token))?;

        Self::parse_user_response(json_obj)
    }

    /// Parse user response JSON into a User struct.
    fn parse_user_response(json_obj: PyObject) -> PyResult<User> {
        let value = pyobject_to_rust_value(&json_obj, "user profile")?;
        serde_json::from_value(value).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to decode user profile: {}",
                e
            ))
        })
    }
}





