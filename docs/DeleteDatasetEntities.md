# DeleteDatasetEntities


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dataset_entity_ids** | **List[str]** |  | 
**remark** | **str** |  | 

## Example

```python
from openapi_client.models.delete_dataset_entities import DeleteDatasetEntities

# TODO update the JSON string below
json = "{}"
# create an instance of DeleteDatasetEntities from a JSON string
delete_dataset_entities_instance = DeleteDatasetEntities.from_json(json)
# print the JSON string representation of the object
print(DeleteDatasetEntities.to_json())

# convert the object into a dict
delete_dataset_entities_dict = delete_dataset_entities_instance.to_dict()
# create an instance of DeleteDatasetEntities from a dict
delete_dataset_entities_from_dict = DeleteDatasetEntities.from_dict(delete_dataset_entities_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


