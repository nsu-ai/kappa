# coding: utf-8

"""
    Kappa Framework SDK, version 1.0.0

"""  # noqa: E501


import unittest

from kf_sdk.api.session_management_api import SessionManagementApi
from kf_sdk.configuration import Configuration
from kf_sdk.api_client import ApiClient
from kf_sdk.models.new_session import NewSession


class TestSessionManagementApi(unittest.TestCase):
    """SessionManagementApi unit test stubs"""

    def setUp(self) -> None:
        self.configuration_users = Configuration(
            host = "https://bigdata.nsu.ru:8460/user-micro-services/v1"
        )
        self.api_client = ApiClient(configuration_users)
        self.api = SessionManagementApi(api_client)
        self.login_id = "anonymous"
        self.passwd = "anonymous"
        self.new_session = NewSession(login_id=self.login_id,passwd=self.passwd)


    def tearDown(self) -> None:
        pass

    def test_get_new_sesion(self) -> None:
        """Test case for new_session_session_login_post

        New Session
        """
        api_response = self.api.get_new_session(self.new_session)
        assert api_response.token!=""

if __name__ == '__main__':
    unittest.main()
