# NewSession


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**login_id** | **str** |  | 
**passwd** | **str** |  | 
**ip_address** | **str** |  | [optional] [default to '127.0.0.1']

## Example

```python
from openapi_client.models.new_session import NewSession

# TODO update the JSON string below
json = "{}"
# create an instance of NewSession from a JSON string
new_session_instance = NewSession.from_json(json)
# print the JSON string representation of the object
print(NewSession.to_json())

# convert the object into a dict
new_session_dict = new_session_instance.to_dict()
# create an instance of NewSession from a dict
new_session_from_dict = NewSession.from_dict(new_session_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


