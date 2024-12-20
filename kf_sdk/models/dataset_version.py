# coding: utf-8

"""
    Kappa-framework data microservices

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
from typing import Optional, Set
from typing_extensions import Self

class DatasetVersion(BaseModel):
    """
    DatasetVersion
    """ # noqa: E501
    id: StrictInt
    user_id: StrictInt = Field(alias="userId")
    dataset_id: StrictInt = Field(alias="datasetId")
    version_availability: Optional[StrictInt] = Field(default=1, alias="versionAvailability")
    version_no: Optional[StrictStr] = Field(default=None, alias="versionNo")
    version_remark: Optional[StrictStr] = Field(default=None, alias="versionRemark")
    created_on: datetime = Field(alias="createdOn")
    modified_on: datetime = Field(alias="modifiedOn")
    __properties: ClassVar[List[str]] = ["id", "userId", "datasetId", "versionAvailability", "versionNo", "versionRemark", "createdOn", "modifiedOn"]

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
        """Create an instance of DatasetVersion from a JSON string"""
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
        # set to None if version_no (nullable) is None
        # and model_fields_set contains the field
        if self.version_no is None and "version_no" in self.model_fields_set:
            _dict['versionNo'] = None

        # set to None if version_remark (nullable) is None
        # and model_fields_set contains the field
        if self.version_remark is None and "version_remark" in self.model_fields_set:
            _dict['versionRemark'] = None

        return _dict

    @classmethod
    def from_dict(cls, obj: Optional[Dict[str, Any]]) -> Optional[Self]:
        """Create an instance of DatasetVersion from a dict"""
        if obj is None:
            return None

        if not isinstance(obj, dict):
            return cls.model_validate(obj)

        _obj = cls.model_validate({
            "id": obj.get("id"),
            "userId": obj.get("userId"),
            "datasetId": obj.get("datasetId"),
            "versionAvailability": obj.get("versionAvailability") if obj.get("versionAvailability") is not None else 1,
            "versionNo": obj.get("versionNo"),
            "versionRemark": obj.get("versionRemark"),
            "createdOn": obj.get("createdOn"),
            "modifiedOn": obj.get("modifiedOn")
        })
        return _obj


