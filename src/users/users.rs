// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use crate::traits::ApiClient;
use crate::models::users_model::User;
use crate::utils::python_json::pyobject_to_rust_value;

/// User profile API helpers.
pub struct Users;

impl Users {
    /// Get current user profile
    ///
    /// Returns a typed :class:`User` mirroring the server JSON.
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

#[cfg(test)]
mod user_profile_tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn load_fixture(name: &str) -> serde_json::Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        let content = fs::read_to_string(path).expect("read fixture");
        serde_json::from_str(&content).expect("parse fixture JSON")
    }

    #[test]
    fn parse_user_without_organization() {
        let user: User = serde_json::from_value(load_fixture("user_profile_no_org.json"))
            .expect("deserialize orgless user");
        assert_eq!(user.user_id, 42);
        assert!(user.org_id.is_none());
        assert!(user.org_details.is_none());
    }

    #[test]
    fn parse_user_with_organization() {
        let user: User = serde_json::from_value(load_fixture("user_profile_with_org.json"))
            .expect("deserialize org user");
        assert_eq!(user.org_id, Some(1));
        let org = user.org_details.expect("org details");
        assert_eq!(org.org_id, 1);
        assert_eq!(org.org_name.as_deref(), Some("SDAML Lab NSU"));
    }
}
