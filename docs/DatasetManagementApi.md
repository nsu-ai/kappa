# openapi_client.DatasetManagementApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_dataset_entities_datasets_dataset_entities_user_id_user_type_id_delete**](DatasetManagementApi.md#delete_dataset_entities_datasets_dataset_entities_user_id_user_type_id_delete) | **DELETE** /datasets/datasetEntities/{user_id}/{user_type_id} | Delete Dataset Entities
[**delete_entity_files_datasets_dataset_entities_files_user_id_user_type_id_delete**](DatasetManagementApi.md#delete_entity_files_datasets_dataset_entities_files_user_id_user_type_id_delete) | **DELETE** /datasets/datasetEntities/files/{user_id}/{user_type_id} | Delete Entity Files
[**get_dataset_entities_datasets_dataset_entities_user_id_user_type_id_dataset_id_get**](DatasetManagementApi.md#get_dataset_entities_datasets_dataset_entities_user_id_user_type_id_dataset_id_get) | **GET** /datasets/datasetEntities/{user_id}/{user_type_id}/{dataset_id} | Get Dataset Entities
[**get_dataset_item_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_get**](DatasetManagementApi.md#get_dataset_item_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_get) | **GET** /datasets/datasetEntities/{user_id}/{user_type_id}/{dataset_id}/{dataset_entity_id} | Get Dataset Item
[**get_entity_file_datasets_dataset_entities_files_user_id_user_type_id_dataset_id_file_id_get**](DatasetManagementApi.md#get_entity_file_datasets_dataset_entities_files_user_id_user_type_id_dataset_id_file_id_get) | **GET** /datasets/datasetEntities/files/{user_id}/{user_type_id}/{dataset_id}/{file_id} | Get Entity File
[**get_thumbnail_datasets_dataset_entities_thumbnail_user_id_user_type_id_dataset_id_file_id_get**](DatasetManagementApi.md#get_thumbnail_datasets_dataset_entities_thumbnail_user_id_user_type_id_dataset_id_file_id_get) | **GET** /datasets/datasetEntities/thumbnail/{user_id}/{user_type_id}/{dataset_id}/{file_id} | Get Thumbnail
[**get_user_datasets_datasets_user_id_user_type_id_get**](DatasetManagementApi.md#get_user_datasets_datasets_user_id_user_type_id_get) | **GET** /datasets/{user_id}/{user_type_id} | Get User Datasets
[**new_dataset_datasets_user_id_user_type_id_post**](DatasetManagementApi.md#new_dataset_datasets_user_id_user_type_id_post) | **POST** /datasets/{user_id}/{user_type_id} | New Dataset
[**new_dataset_entity_datasets_dataset_entities_user_id_user_type_id_post**](DatasetManagementApi.md#new_dataset_entity_datasets_dataset_entities_user_id_user_type_id_post) | **POST** /datasets/datasetEntities/{user_id}/{user_type_id} | New Dataset Entity
[**recover_dataset_entities_datasets_dataset_entities_recover_user_id_user_type_id_post**](DatasetManagementApi.md#recover_dataset_entities_datasets_dataset_entities_recover_user_id_user_type_id_post) | **POST** /datasets/datasetEntities/recover/{user_id}/{user_type_id} | Recover Dataset Entities
[**update_dataset_entity_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_put**](DatasetManagementApi.md#update_dataset_entity_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_put) | **PUT** /datasets/datasetEntities/{user_id}/{user_type_id}/{dataset_id}/{dataset_entity_id} | Update Dataset Entity


# **delete_dataset_entities_datasets_dataset_entities_user_id_user_type_id_delete**
> object delete_dataset_entities_datasets_dataset_entities_user_id_user_type_id_delete(user_id, user_type_id, delete_dataset_entities)

Delete Dataset Entities

### Example


```python
import openapi_client
from openapi_client.models.delete_dataset_entities import DeleteDatasetEntities
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
    api_instance = openapi_client.DatasetManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    delete_dataset_entities = openapi_client.DeleteDatasetEntities() # DeleteDatasetEntities | 

    try:
        # Delete Dataset Entities
        api_response = api_instance.delete_dataset_entities_datasets_dataset_entities_user_id_user_type_id_delete(user_id, user_type_id, delete_dataset_entities)
        print("The response of DatasetManagementApi->delete_dataset_entities_datasets_dataset_entities_user_id_user_type_id_delete:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->delete_dataset_entities_datasets_dataset_entities_user_id_user_type_id_delete: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **delete_dataset_entities** | [**DeleteDatasetEntities**](DeleteDatasetEntities.md)|  | 

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

# **delete_entity_files_datasets_dataset_entities_files_user_id_user_type_id_delete**
> object delete_entity_files_datasets_dataset_entities_files_user_id_user_type_id_delete(user_id, user_type_id, request_body)

Delete Entity Files

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
    api_instance = openapi_client.DatasetManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    request_body = ['request_body_example'] # List[str] | 

    try:
        # Delete Entity Files
        api_response = api_instance.delete_entity_files_datasets_dataset_entities_files_user_id_user_type_id_delete(user_id, user_type_id, request_body)
        print("The response of DatasetManagementApi->delete_entity_files_datasets_dataset_entities_files_user_id_user_type_id_delete:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->delete_entity_files_datasets_dataset_entities_files_user_id_user_type_id_delete: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **request_body** | [**List[str]**](str.md)|  | 

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

# **get_dataset_entities_datasets_dataset_entities_user_id_user_type_id_dataset_id_get**
> List[DatasetEntityDetails] get_dataset_entities_datasets_dataset_entities_user_id_user_type_id_dataset_id_get(user_id, user_type_id, dataset_id)

Get Dataset Entities

### Example


```python
import openapi_client
from openapi_client.models.dataset_entity_details import DatasetEntityDetails
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
    api_instance = openapi_client.DatasetManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    dataset_id = 56 # int | 

    try:
        # Get Dataset Entities
        api_response = api_instance.get_dataset_entities_datasets_dataset_entities_user_id_user_type_id_dataset_id_get(user_id, user_type_id, dataset_id)
        print("The response of DatasetManagementApi->get_dataset_entities_datasets_dataset_entities_user_id_user_type_id_dataset_id_get:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->get_dataset_entities_datasets_dataset_entities_user_id_user_type_id_dataset_id_get: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **dataset_id** | **int**|  | 

### Return type

[**List[DatasetEntityDetails]**](DatasetEntityDetails.md)

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

# **get_dataset_item_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_get**
> DatasetEntityDetails get_dataset_item_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_get(user_id, user_type_id, dataset_id, dataset_entity_id)

Get Dataset Item

### Example


```python
import openapi_client
from openapi_client.models.dataset_entity_details import DatasetEntityDetails
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
    api_instance = openapi_client.DatasetManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    dataset_id = 56 # int | 
    dataset_entity_id = 'dataset_entity_id_example' # str | 

    try:
        # Get Dataset Item
        api_response = api_instance.get_dataset_item_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_get(user_id, user_type_id, dataset_id, dataset_entity_id)
        print("The response of DatasetManagementApi->get_dataset_item_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_get:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->get_dataset_item_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_get: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **dataset_id** | **int**|  | 
 **dataset_entity_id** | **str**|  | 

### Return type

[**DatasetEntityDetails**](DatasetEntityDetails.md)

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

# **get_entity_file_datasets_dataset_entities_files_user_id_user_type_id_dataset_id_file_id_get**
> object get_entity_file_datasets_dataset_entities_files_user_id_user_type_id_dataset_id_file_id_get(user_id, user_type_id, dataset_id, file_id)

Get Entity File

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
    api_instance = openapi_client.DatasetManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    dataset_id = 56 # int | 
    file_id = 'file_id_example' # str | 

    try:
        # Get Entity File
        api_response = api_instance.get_entity_file_datasets_dataset_entities_files_user_id_user_type_id_dataset_id_file_id_get(user_id, user_type_id, dataset_id, file_id)
        print("The response of DatasetManagementApi->get_entity_file_datasets_dataset_entities_files_user_id_user_type_id_dataset_id_file_id_get:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->get_entity_file_datasets_dataset_entities_files_user_id_user_type_id_dataset_id_file_id_get: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **dataset_id** | **int**|  | 
 **file_id** | **str**|  | 

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

# **get_thumbnail_datasets_dataset_entities_thumbnail_user_id_user_type_id_dataset_id_file_id_get**
> object get_thumbnail_datasets_dataset_entities_thumbnail_user_id_user_type_id_dataset_id_file_id_get(user_id, user_type_id, dataset_id, file_id)

Get Thumbnail

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
    api_instance = openapi_client.DatasetManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    dataset_id = 56 # int | 
    file_id = 'file_id_example' # str | 

    try:
        # Get Thumbnail
        api_response = api_instance.get_thumbnail_datasets_dataset_entities_thumbnail_user_id_user_type_id_dataset_id_file_id_get(user_id, user_type_id, dataset_id, file_id)
        print("The response of DatasetManagementApi->get_thumbnail_datasets_dataset_entities_thumbnail_user_id_user_type_id_dataset_id_file_id_get:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->get_thumbnail_datasets_dataset_entities_thumbnail_user_id_user_type_id_dataset_id_file_id_get: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **dataset_id** | **int**|  | 
 **file_id** | **str**|  | 

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

# **get_user_datasets_datasets_user_id_user_type_id_get**
> List[DatasetDetails] get_user_datasets_datasets_user_id_user_type_id_get(user_id, user_type_id)

Get User Datasets

### Example


```python
import openapi_client
from openapi_client.models.dataset_details import DatasetDetails
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
    api_instance = openapi_client.DatasetManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 

    try:
        # Get User Datasets
        api_response = api_instance.get_user_datasets_datasets_user_id_user_type_id_get(user_id, user_type_id)
        print("The response of DatasetManagementApi->get_user_datasets_datasets_user_id_user_type_id_get:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->get_user_datasets_datasets_user_id_user_type_id_get: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 

### Return type

[**List[DatasetDetails]**](DatasetDetails.md)

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

# **new_dataset_datasets_user_id_user_type_id_post**
> object new_dataset_datasets_user_id_user_type_id_post(user_id, user_type_id, new_dataset)

New Dataset

### Example


```python
import openapi_client
from openapi_client.models.new_dataset import NewDataset
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
    api_instance = openapi_client.DatasetManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    new_dataset = openapi_client.NewDataset() # NewDataset | 

    try:
        # New Dataset
        api_response = api_instance.new_dataset_datasets_user_id_user_type_id_post(user_id, user_type_id, new_dataset)
        print("The response of DatasetManagementApi->new_dataset_datasets_user_id_user_type_id_post:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->new_dataset_datasets_user_id_user_type_id_post: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **new_dataset** | [**NewDataset**](NewDataset.md)|  | 

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

# **new_dataset_entity_datasets_dataset_entities_user_id_user_type_id_post**
> object new_dataset_entity_datasets_dataset_entities_user_id_user_type_id_post(user_id, user_type_id, new_dataset_entity, files)

New Dataset Entity

### Example


```python
import openapi_client
from openapi_client.models.new_dataset_entity import NewDatasetEntity
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
    api_instance = openapi_client.DatasetManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    new_dataset_entity = openapi_client.NewDatasetEntity() # NewDatasetEntity | 
    files = None # List[bytearray] | 

    try:
        # New Dataset Entity
        api_response = api_instance.new_dataset_entity_datasets_dataset_entities_user_id_user_type_id_post(user_id, user_type_id, new_dataset_entity, files)
        print("The response of DatasetManagementApi->new_dataset_entity_datasets_dataset_entities_user_id_user_type_id_post:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->new_dataset_entity_datasets_dataset_entities_user_id_user_type_id_post: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **new_dataset_entity** | [**NewDatasetEntity**](NewDatasetEntity.md)|  | 
 **files** | **List[bytearray]**|  | 

### Return type

**object**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: multipart/form-data
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Successful Response |  -  |
**422** | Validation Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **recover_dataset_entities_datasets_dataset_entities_recover_user_id_user_type_id_post**
> object recover_dataset_entities_datasets_dataset_entities_recover_user_id_user_type_id_post(user_id, user_type_id, request_body)

Recover Dataset Entities

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
    api_instance = openapi_client.DatasetManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    request_body = ['request_body_example'] # List[str] | 

    try:
        # Recover Dataset Entities
        api_response = api_instance.recover_dataset_entities_datasets_dataset_entities_recover_user_id_user_type_id_post(user_id, user_type_id, request_body)
        print("The response of DatasetManagementApi->recover_dataset_entities_datasets_dataset_entities_recover_user_id_user_type_id_post:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->recover_dataset_entities_datasets_dataset_entities_recover_user_id_user_type_id_post: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **request_body** | [**List[str]**](str.md)|  | 

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

# **update_dataset_entity_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_put**
> object update_dataset_entity_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_put(user_id, user_type_id, dataset_id, dataset_entity_id, update_dataset_entity, files)

Update Dataset Entity

### Example


```python
import openapi_client
from openapi_client.models.update_dataset_entity import UpdateDatasetEntity
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
    api_instance = openapi_client.DatasetManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    dataset_id = 56 # int | 
    dataset_entity_id = 'dataset_entity_id_example' # str | 
    update_dataset_entity = openapi_client.UpdateDatasetEntity() # UpdateDatasetEntity | 
    files = None # List[bytearray] | 

    try:
        # Update Dataset Entity
        api_response = api_instance.update_dataset_entity_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_put(user_id, user_type_id, dataset_id, dataset_entity_id, update_dataset_entity, files)
        print("The response of DatasetManagementApi->update_dataset_entity_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_put:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->update_dataset_entity_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_put: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **dataset_id** | **int**|  | 
 **dataset_entity_id** | **str**|  | 
 **update_dataset_entity** | [**UpdateDatasetEntity**](UpdateDatasetEntity.md)|  | 
 **files** | **List[bytearray]**|  | 

### Return type

**object**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: multipart/form-data
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Successful Response |  -  |
**422** | Validation Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

