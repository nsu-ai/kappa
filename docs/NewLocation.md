# NewLocation


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**location_name** | **str** |  | 
**user_id** | **int** |  | [optional] 
**lat** | **float** |  | 
**long** | **float** |  | 
**details** | **object** |  | 

## Example

```python
from kf_sdk.models.new_location import NewLocation

# TODO update the JSON string below
json = "{}"
# create an instance of NewLocation from a JSON string
new_location_instance = NewLocation.from_json(json)
# print the JSON string representation of the object
print(NewLocation.to_json())

# convert the object into a dict
new_location_dict = new_location_instance.to_dict()
# create an instance of NewLocation from a dict
new_location_from_dict = NewLocation.from_dict(new_location_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


