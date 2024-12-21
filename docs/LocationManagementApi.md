# kf_sdk.LocationManagementApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_location_locations_user_id_user_type_id_location_id_delete**](LocationManagementApi.md#delete_location_locations_user_id_user_type_id_location_id_delete) | **DELETE** /locations/{user_id}/{user_type_id}/{location_id} | Delete Location
[**get_user_locations_locations_user_id_user_type_id_get**](LocationManagementApi.md#get_user_locations_locations_user_id_user_type_id_get) | **GET** /locations/{user_id}/{user_type_id} | Get User Locations
[**new_location_locations_user_id_user_type_id_post**](LocationManagementApi.md#new_location_locations_user_id_user_type_id_post) | **POST** /locations/{user_id}/{user_type_id} | New Location
[**recover_location_locations_user_id_user_type_id_location_id_put**](LocationManagementApi.md#recover_location_locations_user_id_user_type_id_location_id_put) | **PUT** /locations/{user_id}/{user_type_id}/{location_id} | Recover Location
[**update_location_locations_user_id_user_type_id_put**](LocationManagementApi.md#update_location_locations_user_id_user_type_id_put) | **PUT** /locations/{user_id}/{user_type_id} | Update Location


# **delete_location_locations_user_id_user_type_id_location_id_delete**
> object delete_location_locations_user_id_user_type_id_location_id_delete(user_id, user_type_id, location_id)

Delete Location

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
    api_instance = kf_sdk.LocationManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    location_id = 56 # int | 

    try:
        # Delete Location
        api_response = api_instance.delete_location_locations_user_id_user_type_id_location_id_delete(user_id, user_type_id, location_id)
        print("The response of LocationManagementApi->delete_location_locations_user_id_user_type_id_location_id_delete:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling LocationManagementApi->delete_location_locations_user_id_user_type_id_location_id_delete: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **location_id** | **int**|  | 

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

# **get_user_locations_locations_user_id_user_type_id_get**
> List[LocationDetails] get_user_locations_locations_user_id_user_type_id_get(user_id, user_type_id)

Get User Locations

### Example


```python
import kf_sdk
from kf_sdk.models.location_details import LocationDetails
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
    api_instance = kf_sdk.LocationManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 

    try:
        # Get User Locations
        api_response = api_instance.get_user_locations_locations_user_id_user_type_id_get(user_id, user_type_id)
        print("The response of LocationManagementApi->get_user_locations_locations_user_id_user_type_id_get:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling LocationManagementApi->get_user_locations_locations_user_id_user_type_id_get: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 

### Return type

[**List[LocationDetails]**](LocationDetails.md)

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

# **new_location_locations_user_id_user_type_id_post**
> object new_location_locations_user_id_user_type_id_post(user_id, user_type_id, new_location)

New Location

### Example


```python
import kf_sdk
from kf_sdk.models.new_location import NewLocation
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
    api_instance = kf_sdk.LocationManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    new_location = kf_sdk.NewLocation() # NewLocation | 

    try:
        # New Location
        api_response = api_instance.new_location_locations_user_id_user_type_id_post(user_id, user_type_id, new_location)
        print("The response of LocationManagementApi->new_location_locations_user_id_user_type_id_post:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling LocationManagementApi->new_location_locations_user_id_user_type_id_post: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **new_location** | [**NewLocation**](NewLocation.md)|  | 

### Return type

**object**

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

# **recover_location_locations_user_id_user_type_id_location_id_put**
> object recover_location_locations_user_id_user_type_id_location_id_put(user_id, user_type_id, location_id)

Recover Location

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
    api_instance = kf_sdk.LocationManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    location_id = 56 # int | 

    try:
        # Recover Location
        api_response = api_instance.recover_location_locations_user_id_user_type_id_location_id_put(user_id, user_type_id, location_id)
        print("The response of LocationManagementApi->recover_location_locations_user_id_user_type_id_location_id_put:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling LocationManagementApi->recover_location_locations_user_id_user_type_id_location_id_put: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **location_id** | **int**|  | 

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

# **update_location_locations_user_id_user_type_id_put**
> object update_location_locations_user_id_user_type_id_put(user_id, user_type_id, update_location)

Update Location

### Example


```python
import kf_sdk
from kf_sdk.models.update_location import UpdateLocation
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
    api_instance = kf_sdk.LocationManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    update_location = kf_sdk.UpdateLocation() # UpdateLocation | 

    try:
        # Update Location
        api_response = api_instance.update_location_locations_user_id_user_type_id_put(user_id, user_type_id, update_location)
        print("The response of LocationManagementApi->update_location_locations_user_id_user_type_id_put:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling LocationManagementApi->update_location_locations_user_id_user_type_id_put: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **update_location** | [**UpdateLocation**](UpdateLocation.md)|  | 

### Return type

**object**

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

