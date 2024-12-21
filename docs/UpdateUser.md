# UpdateUser


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**first_name** | **str** |  | [optional] 
**middle_name** | **str** |  | [optional] 
**last_name** | **str** |  | [optional] 
**passwd** | **str** |  | [optional] 
**org_id** | **int** |  | [optional] 

## Example

```python
from kf_sdk.models.update_user import UpdateUser

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateUser from a JSON string
update_user_instance = UpdateUser.from_json(json)
# print the JSON string representation of the object
print(UpdateUser.to_json())

# convert the object into a dict
update_user_dict = update_user_instance.to_dict()
# create an instance of UpdateUser from a dict
update_user_from_dict = UpdateUser.from_dict(update_user_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


