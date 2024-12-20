# openapi_client.SessionManagementApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_session_session_logout_user_id_user_type_id_delete**](SessionManagementApi.md#delete_session_session_logout_user_id_user_type_id_delete) | **DELETE** /session/logout/{user_id}/{user_type_id} | Delete Session
[**new_session_session_login_post**](SessionManagementApi.md#new_session_session_login_post) | **POST** /session/login | New Session


# **delete_session_session_logout_user_id_user_type_id_delete**
> object delete_session_session_logout_user_id_user_type_id_delete(user_id, user_type_id)

Delete Session

### Example


```python
import openapi_client
from openapi_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = openapi_client.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with openapi_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = openapi_client.SessionManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 

    try:
        # Delete Session
        api_response = api_instance.delete_session_session_logout_user_id_user_type_id_delete(user_id, user_type_id)
        print("The response of SessionManagementApi->delete_session_session_logout_user_id_user_type_id_delete:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling SessionManagementApi->delete_session_session_logout_user_id_user_type_id_delete: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 

### Return type

**object**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Successful Response |  -  |
**422** | Validation Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **new_session_session_login_post**
> UserDetails new_session_session_login_post(new_session)

New Session

### Example


```python
import openapi_client
from openapi_client.models.new_session import NewSession
from openapi_client.models.user_details import UserDetails
from openapi_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = openapi_client.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with openapi_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = openapi_client.SessionManagementApi(api_client)
    new_session = openapi_client.NewSession() # NewSession | 

    try:
        # New Session
        api_response = api_instance.new_session_session_login_post(new_session)
        print("The response of SessionManagementApi->new_session_session_login_post:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling SessionManagementApi->new_session_session_login_post: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **new_session** | [**NewSession**](NewSession.md)|  | 

### Return type

[**UserDetails**](UserDetails.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Successful Response |  -  |
**422** | Validation Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

