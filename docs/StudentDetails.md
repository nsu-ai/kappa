# StudentDetails


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**student_id** | **int** |  | 
**student_name** | **str** |  | 

## Example

```python
from kf_sdk.models.student_details import StudentDetails

# TODO update the JSON string below
json = "{}"
# create an instance of StudentDetails from a JSON string
student_details_instance = StudentDetails.from_json(json)
# print the JSON string representation of the object
print(StudentDetails.to_json())

# convert the object into a dict
student_details_dict = student_details_instance.to_dict()
# create an instance of StudentDetails from a dict
student_details_from_dict = StudentDetails.from_dict(student_details_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


