# NewDataset


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**user_id** | **int** |  | [optional] [default to 0]
**dataset_name** | **str** |  | 
**dataset_type** | **int** |  | [optional] 
**dataset_short_info** | **str** |  | [optional] 
**dataset_tags** | **str** |  | [optional] 

## Example

```python
from kf_sdk.models.new_dataset import NewDataset

# TODO update the JSON string below
json = "{}"
# create an instance of NewDataset from a JSON string
new_dataset_instance = NewDataset.from_json(json)
# print the JSON string representation of the object
print(NewDataset.to_json())

# convert the object into a dict
new_dataset_dict = new_dataset_instance.to_dict()
# create an instance of NewDataset from a dict
new_dataset_from_dict = NewDataset.from_dict(new_dataset_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


