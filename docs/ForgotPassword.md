# ForgotPassword


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**login_id** | **str** |  | 

## Example

```python
from kf_sdk.models.forgot_password import ForgotPassword

# TODO update the JSON string below
json = "{}"
# create an instance of ForgotPassword from a JSON string
forgot_password_instance = ForgotPassword.from_json(json)
# print the JSON string representation of the object
print(ForgotPassword.to_json())

# convert the object into a dict
forgot_password_dict = forgot_password_instance.to_dict()
# create an instance of ForgotPassword from a dict
forgot_password_from_dict = ForgotPassword.from_dict(forgot_password_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


