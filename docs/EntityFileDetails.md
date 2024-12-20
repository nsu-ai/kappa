# EntityFileDetails


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**file_id** | **str** |  | 
**file_name** | **str** |  | 
**content_type** | **str** |  | 
**thumbnail_file** | **str** |  | 

## Example

```python
from openapi_client.models.entity_file_details import EntityFileDetails

# TODO update the JSON string below
json = "{}"
# create an instance of EntityFileDetails from a JSON string
entity_file_details_instance = EntityFileDetails.from_json(json)
# print the JSON string representation of the object
print(EntityFileDetails.to_json())

# convert the object into a dict
entity_file_details_dict = entity_file_details_instance.to_dict()
# create an instance of EntityFileDetails from a dict
entity_file_details_from_dict = EntityFileDetails.from_dict(entity_file_details_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


