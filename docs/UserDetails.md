# UserDetails


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**user_name** | **str** |  | 
**first_name** | **str** |  | 
**middle_name** | **str** |  | 
**last_name** | **str** |  | 
**email** | **str** |  | 
**user_type_id** | **int** |  | 
**org_id** | **int** |  | 
**user_id** | **int** |  | 
**user_type** | **str** |  | [optional] 
**org_name** | **str** |  | [optional] 
**token** | **str** |  | [optional] 
**token_expiry_at** | **str** |  | [optional] 
**profile_pic** | **str** |  | [optional] 

## Example

```python
from openapi_client.models.user_details import UserDetails

# TODO update the JSON string below
json = "{}"
# create an instance of UserDetails from a JSON string
user_details_instance = UserDetails.from_json(json)
# print the JSON string representation of the object
print(UserDetails.to_json())

# convert the object into a dict
user_details_dict = user_details_instance.to_dict()
# create an instance of UserDetails from a dict
user_details_from_dict = UserDetails.from_dict(user_details_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


