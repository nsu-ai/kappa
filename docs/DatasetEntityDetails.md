# DatasetEntityDetails


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dataset_id** | **int** |  | [optional] 
**user_id** | **int** |  | [optional] 
**ds_entity_name** | **str** |  | 
**entity_source** | **str** |  | 
**collected_on** | **datetime** |  | 
**labeling_algo** | **str** |  | 
**ds_entity_info** | **object** |  | 
**location_id** | **int** |  | 
**ds_entity_id** | **str** |  | 
**ds_entity_status** | **int** |  | 
**entity_status_intrep** | **str** |  | [optional] 
**entity_verification_status** | **int** |  | 
**verification_status_interp** | **str** |  | [optional] 
**created_on** | **datetime** |  | 
**modified_on** | **datetime** |  | 
**location_details** | [**LocationDetails**](LocationDetails.md) |  | 
**files** | [**List[EntityFileDetails]**](EntityFileDetails.md) |  | [optional] 

## Example

```python
from openapi_client.models.dataset_entity_details import DatasetEntityDetails

# TODO update the JSON string below
json = "{}"
# create an instance of DatasetEntityDetails from a JSON string
dataset_entity_details_instance = DatasetEntityDetails.from_json(json)
# print the JSON string representation of the object
print(DatasetEntityDetails.to_json())

# convert the object into a dict
dataset_entity_details_dict = dataset_entity_details_instance.to_dict()
# create an instance of DatasetEntityDetails from a dict
dataset_entity_details_from_dict = DatasetEntityDetails.from_dict(dataset_entity_details_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


