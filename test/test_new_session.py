# coding: utf-8

"""
    Kappa Framework SDK, version 1.0.0

"""  # noqa: E501


import unittest

from kf_sdk.models.new_session import NewSession

class TestNewSession(unittest.TestCase):
    """NewSession unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> NewSession:
        """Test NewSession
            include_optional is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        model = NewSession()
        if include_optional:
            return NewSession(
                login_id = 'anonymous',
                passwd = 'anonymous',
                ip_address = '127.0.0.1'
            )
        else:
            return NewSession(
                login_id = 'anonymous',
                passwd = 'anonymous',
        )

    def testNewSession(self):
        """Test NewSession"""
        inst_req_only = self.make_instance(include_optional=False)
        inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()
