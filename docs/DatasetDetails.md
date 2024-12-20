# DatasetDetails


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**user_id** | **int** |  | [optional] [default to 0]
**dataset_name** | **str** |  | 
**dataset_type** | **int** |  | [optional] 
**dataset_short_info** | **str** |  | [optional] 
**dataset_tags** | **str** |  | [optional] 
**dataset_id** | **int** |  | 
**dataset_type_interp** | **str** |  | [optional] 
**dataset_status** | **int** |  | 
**dataset_status_interp** | **str** |  | [optional] 
**created_on** | **datetime** |  | 
**modified_on** | **datetime** |  | 

## Example

```python
from openapi_client.models.dataset_details import DatasetDetails

# TODO update the JSON string below
json = "{}"
# create an instance of DatasetDetails from a JSON string
dataset_details_instance = DatasetDetails.from_json(json)
# print the JSON string representation of the object
print(DatasetDetails.to_json())

# convert the object into a dict
dataset_details_dict = dataset_details_instance.to_dict()
# create an instance of DatasetDetails from a dict
dataset_details_from_dict = DatasetDetails.from_dict(dataset_details_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


