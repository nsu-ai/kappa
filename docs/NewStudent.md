# NewStudent


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**first_name** | **str** |  | 
**middle_name** | **str** |  | 
**last_name** | **str** |  | 
**email** | **str** |  | 

## Example

```python
from kf_sdk.models.new_student import NewStudent

# TODO update the JSON string below
json = "{}"
# create an instance of NewStudent from a JSON string
new_student_instance = NewStudent.from_json(json)
# print the JSON string representation of the object
print(NewStudent.to_json())

# convert the object into a dict
new_student_dict = new_student_instance.to_dict()
# create an instance of NewStudent from a dict
new_student_from_dict = NewStudent.from_dict(new_student_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


