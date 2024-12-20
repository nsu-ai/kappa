# coding: utf-8

"""
    Kappa-framework user microservices

    No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)

    The version of the OpenAPI document: 0.1.0
    Generated by OpenAPI Generator (https://openapi-generator.tech)

    Do not edit the class manually.
"""  # noqa: E501


from __future__ import annotations
import pprint
import re  # noqa: F401
import json

from datetime import datetime
from pydantic import BaseModel, ConfigDict, Field, StrictInt, StrictStr
from typing import Any, ClassVar, Dict, List, Optional
from kf_sdk.models.organization_details import OrganizationDetails
from kf_sdk.models.user_type_details import UserTypeDetails
from typing import Optional, Set
from typing_extensions import Self

class UserDetails(BaseModel):
    """
    UserDetails
    """ # noqa: E501
    user_id: StrictInt = Field(alias="userId")
    user_name: StrictStr = Field(alias="userName")
    first_name: StrictStr = Field(alias="firstName")
    middle_name: Optional[StrictStr] = Field(alias="middleName")
    last_name: StrictStr = Field(alias="lastName")
    email: StrictStr
    user_type_id: StrictInt = Field(alias="userTypeId")
    org_id: StrictInt = Field(alias="orgId")
    user_type_details: Optional[UserTypeDetails] = Field(default=None, alias="userTypeDetails")
    org_details: Optional[OrganizationDetails] = Field(default=None, alias="orgDetails")
    profile_pic: Optional[StrictStr] = Field(default=None, alias="profilePic")
    token: Optional[StrictStr] = None
    token_expiry_date: Optional[datetime] = Field(default=None, alias="tokenExpiryDate")
    __properties: ClassVar[List[str]] = ["userId", "userName", "firstName", "middleName", "lastName", "email", "userTypeId", "orgId", "userTypeDetails", "orgDetails", "profilePic", "token", "tokenExpiryDate"]

    model_config = ConfigDict(
        populate_by_name=True,
        validate_assignment=True,
        protected_namespaces=(),
    )


    def to_str(self) -> str:
        """Returns the string representation of the model using alias"""
        return pprint.pformat(self.model_dump(by_alias=True))

    def to_json(self) -> str:
        """Returns the JSON representation of the model using alias"""
        # TODO: pydantic v2: use .model_dump_json(by_alias=True, exclude_unset=True) instead
        return json.dumps(self.to_dict())

    @classmethod
    def from_json(cls, json_str: str) -> Optional[Self]:
        """Create an instance of UserDetails from a JSON string"""
        return cls.from_dict(json.loads(json_str))

    def to_dict(self) -> Dict[str, Any]:
        """Return the dictionary representation of the model using alias.

        This has the following differences from calling pydantic's
        `self.model_dump(by_alias=True)`:

        * `None` is only added to the output dict for nullable fields that
          were set at model initialization. Other fields with value `None`
          are ignored.
        """
        excluded_fields: Set[str] = set([
        ])

        _dict = self.model_dump(
            by_alias=True,
            exclude=excluded_fields,
            exclude_none=True,
        )
        # override the default output from pydantic by calling `to_dict()` of user_type_details
        if self.user_type_details:
            _dict['userTypeDetails'] = self.user_type_details.to_dict()
        # override the default output from pydantic by calling `to_dict()` of org_details
        if self.org_details:
            _dict['orgDetails'] = self.org_details.to_dict()
        # set to None if middle_name (nullable) is None
        # and model_fields_set contains the field
        if self.middle_name is None and "middle_name" in self.model_fields_set:
            _dict['middleName'] = None

        # set to None if user_type_details (nullable) is None
        # and model_fields_set contains the field
        if self.user_type_details is None and "user_type_details" in self.model_fields_set:
            _dict['userTypeDetails'] = None

        # set to None if org_details (nullable) is None
        # and model_fields_set contains the field
        if self.org_details is None and "org_details" in self.model_fields_set:
            _dict['orgDetails'] = None

        # set to None if profile_pic (nullable) is None
        # and model_fields_set contains the field
        if self.profile_pic is None and "profile_pic" in self.model_fields_set:
            _dict['profilePic'] = None

        # set to None if token (nullable) is None
        # and model_fields_set contains the field
        if self.token is None and "token" in self.model_fields_set:
            _dict['token'] = None

        # set to None if token_expiry_date (nullable) is None
        # and model_fields_set contains the field
        if self.token_expiry_date is None and "token_expiry_date" in self.model_fields_set:
            _dict['tokenExpiryDate'] = None

        return _dict

    @classmethod
    def from_dict(cls, obj: Optional[Dict[str, Any]]) -> Optional[Self]:
        """Create an instance of UserDetails from a dict"""
        if obj is None:
            return None

        if not isinstance(obj, dict):
            return cls.model_validate(obj)

        _obj = cls.model_validate({
            "userId": obj.get("userId"),
            "userName": obj.get("userName"),
            "firstName": obj.get("firstName"),
            "middleName": obj.get("middleName"),
            "lastName": obj.get("lastName"),
            "email": obj.get("email"),
            "userTypeId": obj.get("userTypeId"),
            "orgId": obj.get("orgId"),
            "userTypeDetails": UserTypeDetails.from_dict(obj["userTypeDetails"]) if obj.get("userTypeDetails") is not None else None,
            "orgDetails": OrganizationDetails.from_dict(obj["orgDetails"]) if obj.get("orgDetails") is not None else None,
            "profilePic": obj.get("profilePic"),
            "token": obj.get("token"),
            "tokenExpiryDate": obj.get("tokenExpiryDate")
        })
        return _obj


