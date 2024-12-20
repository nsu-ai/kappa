# openapi_client.DashboardDataApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_dashboard_data_dashboard_user_id_user_type_id_get**](DashboardDataApi.md#get_dashboard_data_dashboard_user_id_user_type_id_get) | **GET** /dashboard/{user_id}/{user_type_id} | Get Dashboard Data


# **get_dashboard_data_dashboard_user_id_user_type_id_get**
> object get_dashboard_data_dashboard_user_id_user_type_id_get(user_id, user_type_id)

Get Dashboard Data

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
    api_instance = openapi_client.DashboardDataApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 

    try:
        # Get Dashboard Data
        api_response = api_instance.get_dashboard_data_dashboard_user_id_user_type_id_get(user_id, user_type_id)
        print("The response of DashboardDataApi->get_dashboard_data_dashboard_user_id_user_type_id_get:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DashboardDataApi->get_dashboard_data_dashboard_user_id_user_type_id_get: %s\n" % e)
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

