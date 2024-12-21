# NewDatasetEntity


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dataset_id** | **int** |  | 
**user_id** | **int** |  | 
**ds_entity_name** | **str** |  | 
**entity_source** | **str** |  | [optional] 
**collected_on** | **datetime** |  | 
**labeling_algo** | **str** |  | [optional] 
**ds_entity_info** | **object** |  | 
**location_id** | **int** |  | 

## Example

```python
from kf_sdk.models.new_dataset_entity import NewDatasetEntity

# TODO update the JSON string below
json = "{}"
# create an instance of NewDatasetEntity from a JSON string
new_dataset_entity_instance = NewDatasetEntity.from_json(json)
# print the JSON string representation of the object
print(NewDatasetEntity.to_json())

# convert the object into a dict
new_dataset_entity_dict = new_dataset_entity_instance.to_dict()
# create an instance of NewDatasetEntity from a dict
new_dataset_entity_from_dict = NewDatasetEntity.from_dict(new_dataset_entity_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


