# openapi_client.UsersManagementApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**forgot_password_users_forgot_password_post**](UsersManagementApi.md#forgot_password_users_forgot_password_post) | **POST** /users/forgotPassword | Forgot Password
[**get_profile_picture_users_profile_pic_user_id_user_type_id_get**](UsersManagementApi.md#get_profile_picture_users_profile_pic_user_id_user_type_id_get) | **GET** /users/profilePic/{user_id}/{user_type_id} | Get Profile Picture
[**get_user_locations_users_students_user_id_user_type_id_get**](UsersManagementApi.md#get_user_locations_users_students_user_id_user_type_id_get) | **GET** /users/students/{user_id}/{user_type_id} | Get User Locations
[**new_student_users_students_user_id_user_type_id_post**](UsersManagementApi.md#new_student_users_students_user_id_user_type_id_post) | **POST** /users/students/{user_id}/{user_type_id} | New Student
[**reset_password_users_reset_password_post**](UsersManagementApi.md#reset_password_users_reset_password_post) | **POST** /users/resetPassword | Reset Password
[**update_profile_picture_users_profile_pic_user_id_user_type_id_post**](UsersManagementApi.md#update_profile_picture_users_profile_pic_user_id_user_type_id_post) | **POST** /users/profilePic/{user_id}/{user_type_id} | Update Profile Picture
[**update_user_profile_users_user_id_user_type_id_put**](UsersManagementApi.md#update_user_profile_users_user_id_user_type_id_put) | **PUT** /users/{user_id}/{user_type_id} | Update User Profile
[**validate_user_sign_up_users_validate_user_id_user_type_id_post**](UsersManagementApi.md#validate_user_sign_up_users_validate_user_id_user_type_id_post) | **POST** /users/validate/{user_id}/{user_type_id} | Validate User Sign Up


# **forgot_password_users_forgot_password_post**
> object forgot_password_users_forgot_password_post(forgot_password)

Forgot Password

### Example


```python
import openapi_client
from openapi_client.models.forgot_password import ForgotPassword
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
    api_instance = openapi_client.UsersManagementApi(api_client)
    forgot_password = openapi_client.ForgotPassword() # ForgotPassword | 

    try:
        # Forgot Password
        api_response = api_instance.forgot_password_users_forgot_password_post(forgot_password)
        print("The response of UsersManagementApi->forgot_password_users_forgot_password_post:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling UsersManagementApi->forgot_password_users_forgot_password_post: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **forgot_password** | [**ForgotPassword**](ForgotPassword.md)|  | 

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

# **get_profile_picture_users_profile_pic_user_id_user_type_id_get**
> object get_profile_picture_users_profile_pic_user_id_user_type_id_get(user_id, user_type_id)

Get Profile Picture

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
    api_instance = openapi_client.UsersManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 

    try:
        # Get Profile Picture
        api_response = api_instance.get_profile_picture_users_profile_pic_user_id_user_type_id_get(user_id, user_type_id)
        print("The response of UsersManagementApi->get_profile_picture_users_profile_pic_user_id_user_type_id_get:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling UsersManagementApi->get_profile_picture_users_profile_pic_user_id_user_type_id_get: %s\n" % e)
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

# **get_user_locations_users_students_user_id_user_type_id_get**
> List[StudentDetails] get_user_locations_users_students_user_id_user_type_id_get(user_id, user_type_id)

Get User Locations

### Example


```python
import openapi_client
from openapi_client.models.student_details import StudentDetails
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
    api_instance = openapi_client.UsersManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 

    try:
        # Get User Locations
        api_response = api_instance.get_user_locations_users_students_user_id_user_type_id_get(user_id, user_type_id)
        print("The response of UsersManagementApi->get_user_locations_users_students_user_id_user_type_id_get:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling UsersManagementApi->get_user_locations_users_students_user_id_user_type_id_get: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 

### Return type

[**List[StudentDetails]**](StudentDetails.md)

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

# **new_student_users_students_user_id_user_type_id_post**
> object new_student_users_students_user_id_user_type_id_post(user_id, user_type_id, new_student)

New Student

### Example


```python
import openapi_client
from openapi_client.models.new_student import NewStudent
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
    api_instance = openapi_client.UsersManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    new_student = openapi_client.NewStudent() # NewStudent | 

    try:
        # New Student
        api_response = api_instance.new_student_users_students_user_id_user_type_id_post(user_id, user_type_id, new_student)
        print("The response of UsersManagementApi->new_student_users_students_user_id_user_type_id_post:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling UsersManagementApi->new_student_users_students_user_id_user_type_id_post: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **new_student** | [**NewStudent**](NewStudent.md)|  | 

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

# **reset_password_users_reset_password_post**
> object reset_password_users_reset_password_post(reset_password)

Reset Password

### Example


```python
import openapi_client
from openapi_client.models.reset_password import ResetPassword
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
    api_instance = openapi_client.UsersManagementApi(api_client)
    reset_password = openapi_client.ResetPassword() # ResetPassword | 

    try:
        # Reset Password
        api_response = api_instance.reset_password_users_reset_password_post(reset_password)
        print("The response of UsersManagementApi->reset_password_users_reset_password_post:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling UsersManagementApi->reset_password_users_reset_password_post: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **reset_password** | [**ResetPassword**](ResetPassword.md)|  | 

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

# **update_profile_picture_users_profile_pic_user_id_user_type_id_post**
> object update_profile_picture_users_profile_pic_user_id_user_type_id_post(user_id, user_type_id, pic)

Update Profile Picture

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
    api_instance = openapi_client.UsersManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    pic = None # bytearray | 

    try:
        # Update Profile Picture
        api_response = api_instance.update_profile_picture_users_profile_pic_user_id_user_type_id_post(user_id, user_type_id, pic)
        print("The response of UsersManagementApi->update_profile_picture_users_profile_pic_user_id_user_type_id_post:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling UsersManagementApi->update_profile_picture_users_profile_pic_user_id_user_type_id_post: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **pic** | **bytearray**|  | 

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

# **update_user_profile_users_user_id_user_type_id_put**
> object update_user_profile_users_user_id_user_type_id_put(user_id, user_type_id, update_user)

Update User Profile

### Example


```python
import openapi_client
from openapi_client.models.update_user import UpdateUser
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
    api_instance = openapi_client.UsersManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 
    update_user = openapi_client.UpdateUser() # UpdateUser | 

    try:
        # Update User Profile
        api_response = api_instance.update_user_profile_users_user_id_user_type_id_put(user_id, user_type_id, update_user)
        print("The response of UsersManagementApi->update_user_profile_users_user_id_user_type_id_put:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling UsersManagementApi->update_user_profile_users_user_id_user_type_id_put: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **int**|  | 
 **user_type_id** | **int**|  | 
 **update_user** | [**UpdateUser**](UpdateUser.md)|  | 

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

# **validate_user_sign_up_users_validate_user_id_user_type_id_post**
> object validate_user_sign_up_users_validate_user_id_user_type_id_post(user_id, user_type_id)

Validate User Sign Up

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
    api_instance = openapi_client.UsersManagementApi(api_client)
    user_id = 56 # int | 
    user_type_id = 56 # int | 

    try:
        # Validate User Sign Up
        api_response = api_instance.validate_user_sign_up_users_validate_user_id_user_type_id_post(user_id, user_type_id)
        print("The response of UsersManagementApi->validate_user_sign_up_users_validate_user_id_user_type_id_post:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling UsersManagementApi->validate_user_sign_up_users_validate_user_id_user_type_id_post: %s\n" % e)
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

