# coding: utf-8

"""
    Kappa Framework SDK, version 1.0.0

"""  # noqa: E501


import unittest

from kf_sdk.api.session_management_api import SessionManagementApi
from kf_sdk import Configuration


class TestSessionManagementApi(unittest.TestCase):
    """SessionManagementApi unit test stubs"""

    def setUp(self) -> None:
        self.configuration_users = kf_sdk.Configuration(
            host = "https://bigdata.nsu.ru:8460/user-micro-services/v1"
        )
        self.api_client = kf_sdk.ApiClient(configuration_users)
        self.api = kf_sdk.SessionManagementApi(api_client)
        login_id = "anonymous"
        passwd = "anonymous"
        new_session = NewSession(login_id=login_id,passwd=passwd)


    def tearDown(self) -> None:
        pass

    def test_get_new_sesion(self) -> None:
        """Test case for new_session_session_login_post

        New Session
        """
        api_response = self.api.get_new_session(new_session)
        assert api_response.token!=""

if __name__ == '__main__':
    unittest.main()
