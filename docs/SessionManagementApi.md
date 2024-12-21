# kf_sdk.SessionManagementApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_session_session_logout_user_id_user_type_id_delete**](SessionManagementApi.md#delete_session_session_logout_user_id_user_type_id_delete) | **DELETE** /session/logout/{user_id}/{user_type_id} | Delete Session
[**get_new_session**](SessionManagementApi.md#get_new_session) | **POST** /session/login | New Session


# **delete_session_session_logout_user_id_user_type_id_delete**
> object delete_session_session_logout_user_id_user_type_id_delete(user_id, user_type_id)

Delete Session

### Example


```python
import kf_sdk
from kf_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kf_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with kf_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kf_sdk.SessionManagementApi(api_client)
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

# **get_new_session**
> UserDetails get_new_session(new_session)

New Session

### Example


```python
import kf_sdk
from kf_sdk.models.new_session import NewSession
from kf_sdk.models.user_details import UserDetails
from kf_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kf_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with kf_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kf_sdk.SessionManagementApi(api_client)
    new_session = kf_sdk.NewSession() # NewSession | 

    try:
        # New Session
        api_response = api_instance.get_new_session(new_session)
        print("The response of SessionManagementApi->get_new_session:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling SessionManagementApi->get_new_session: %s\n" % e)
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

