# UpdateDatasetEntity


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ds_entity_name** | **str** |  | [optional] 
**entity_source** | **str** |  | [optional] 
**collected_on** | **datetime** |  | [optional] 
**labeling_algo** | **str** |  | [optional] 
**ds_entity_info** | **object** |  | [optional] 
**location_id** | **int** |  | [optional] 
**remark** | **str** |  | 

## Example

```python
from openapi_client.models.update_dataset_entity import UpdateDatasetEntity

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateDatasetEntity from a JSON string
update_dataset_entity_instance = UpdateDatasetEntity.from_json(json)
# print the JSON string representation of the object
print(UpdateDatasetEntity.to_json())

# convert the object into a dict
update_dataset_entity_dict = update_dataset_entity_instance.to_dict()
# create an instance of UpdateDatasetEntity from a dict
update_dataset_entity_from_dict = UpdateDatasetEntity.from_dict(update_dataset_entity_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


