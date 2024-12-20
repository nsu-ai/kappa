import kf_sdk
from kf_sdk.rest import ApiException
from kf_sdk.models import NewSession
from pprint import pprint

# Defining the host is optional and defaults to http://localhost/user-micro-services/v1
# See configuration.py for a list of all supported configuration parameters.
configuration_users = kf_sdk.Configuration(
    host = "https://bigdata.nsu.ru:8460/user-micro-services/v1"
)
# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure Bearer authorization: HTTPBearer
#configuration = kappa_users.Configuration(
#    access_token = os.environ["BEARER_TOKEN"]
#)


# Enter a context with an instance of the API client
with kf_sdk.ApiClient(configuration_users) as api_client:
    # Create an instance of the API class
    api_instance = kf_sdk.SessionManagementApi(api_client)
    login_id = "anonymous"
    passwd = "anonymous"
    new_session = NewSession(login_id=login_id,passwd=passwd)

    try:
        # New Session
        api_response = api_instance.get_new_session(new_session)
        token = api_response.token
        user_id = api_response.user_id
        user_type_id = api_response.user_type_id
        print("The response of SessionManagementApi->new_session_session_new_post:\n")
        pprint(api_response)
    except ApiException as e:
        print("Exception when calling ExpertManagementApi->new_session_session_new_post: %s\n" % e)

configuration_data = kf_sdk.Configuration(
    host = "https://bigdata.nsu.ru:8460/data-micro-services/v1",
    access_token = token
)


with kf_sdk.ApiClient(configuration_data) as api_client:
    # Create an instance of the API class
    api_instance = kf_sdk.DatasetManagementApi(api_client)
    dataset_id = 582
    dataset_version_no = '1.0.0'

    try:
        # Get Dataset Version
        api_response = api_instance.get_dataset_version_archive_datasets_versions_archive_user_id_user_type_id_dataset_id_dataset_version_no_get_without_preload_content(user_id, user_type_id, dataset_id, dataset_version_no)
        pprint(api_response)
        with open("tmp.zip","wb") as f:
            f.write(api_response.data)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->get_dataset_version_datasets_versions_user_id_user_type_id_dataset_id_dataset_version_no_get: %s\n" % e)
