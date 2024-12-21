# LocationDetails


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**location_name** | **str** |  | 
**user_id** | **int** |  | [optional] 
**lat** | **float** |  | 
**long** | **float** |  | 
**details** | **object** |  | 
**location_id** | **int** |  | 

## Example

```python
from kf_sdk.models.location_details import LocationDetails

# TODO update the JSON string below
json = "{}"
# create an instance of LocationDetails from a JSON string
location_details_instance = LocationDetails.from_json(json)
# print the JSON string representation of the object
print(LocationDetails.to_json())

# convert the object into a dict
location_details_dict = location_details_instance.to_dict()
# create an instance of LocationDetails from a dict
location_details_from_dict = LocationDetails.from_dict(location_details_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


