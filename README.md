# kf_sdk

Kappa Framwork Python SDK for Kappa v.1.0.0

- API version: 0.1.0
- Package version: 1.0.0

## Requirements.

Python 3.8+

## Installation & Usage
### pip install

If the python package is hosted on a repository, you can install directly using:

```sh
pip install kf_sdk
```
(you may need to run `pip` with root permission: `sudo pip install kf_sdk`)

Then import the package:
```python
import kf_sdk
```

### Setuptools

Install via [Setuptools](http://pypi.python.org/pypi/setuptools).

```sh
python setup.py install --user
```
(or `sudo python setup.py install` to install the package for all users)

Then import the package:
```python
import kf_sdk
```

### Tests

Execute `pytest` to run the tests.

## Getting Started

Please follow the [installation procedure](#installation--usage) and then run the following:

```python

import kf_sdk
from kf_sdk.rest import ApiException
from kf_sdk.models import NewSession
from pprint import pprint

# Defining the host is optional and defaults to http://localhost/user-micro-services/v1
# See configuration.py for a list of all supported configuration parameters.
configuration_users = kf_sdk.Configuration(
    host = "https://bigdata.nsu.ru:8460/user-micro-services/v1"
)
# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure Bearer authorization: HTTPBearer
#configuration = kappa_users.Configuration(
#    access_token = os.environ["BEARER_TOKEN"]
#)


# Enter a context with an instance of the API client
with kf_sdk.ApiClient(configuration_users) as api_client:
    # Create an instance of the API class
    api_instance = kf_sdk.SessionManagementApi(api_client)
    login_id = "anonymous"
    passwd = "anonymous"
    new_session = NewSession(login_id=login_id,passwd=passwd)

    try:
        # New Session
        api_response = api_instance.get_new_session(new_session)
        token = api_response.token
        user_id = api_response.user_id
        user_type_id = api_response.user_type_id
        print("The response of SessionManagementApi->new_session_session_new_post:\n")
        pprint(api_response)
    except ApiException as e:
        print("Exception when calling ExpertManagementApi->new_session_session_new_post: %s\n" % e)

configuration_data = kf_sdk.Configuration(
    host = "https://bigdata.nsu.ru:8460/data-micro-services/v1",
    access_token = token
)


with kf_sdk.ApiClient(configuration_data) as api_client:
    # Create an instance of the API class
    api_instance = kf_sdk.DatasetManagementApi(api_client)
    dataset_id = 582
    dataset_version_no = '1.0.0'

    try:
        # Get Dataset Version
        api_response = api_instance.get_dataset_version_archive_datasets_versions_archive_user_id_user_type_id_dataset_id_dataset_version_no_get_without_preload_content(user_id, user_type_id, dataset_id, dataset_version_no)
        pprint(api_response)
        with open("tmp.zip","wb") as f:
            f.write(api_response.data)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->get_dataset_version_datasets_versions_user_id_user_type_id_dataset_id_dataset_version_no_get: %s\n" % e)

```

## Documentation for API Endpoints

All URIs are relative to *http://localhost*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*DashboardDataApi* | [**get_dashboard_data_dashboard_user_id_user_type_id_get**](docs/DashboardDataApi.md#get_dashboard_data_dashboard_user_id_user_type_id_get) | **GET** /dashboard/{user_id}/{user_type_id} | Get Dashboard Data
*DatasetManagementApi* | [**delete_dataset_entities_datasets_dataset_entities_user_id_user_type_id_delete**](docs/DatasetManagementApi.md#delete_dataset_entities_datasets_dataset_entities_user_id_user_type_id_delete) | **DELETE** /datasets/datasetEntities/{user_id}/{user_type_id} | Delete Dataset Entities
*DatasetManagementApi* | [**delete_entity_files_datasets_dataset_entities_files_user_id_user_type_id_delete**](docs/DatasetManagementApi.md#delete_entity_files_datasets_dataset_entities_files_user_id_user_type_id_delete) | **DELETE** /datasets/datasetEntities/files/{user_id}/{user_type_id} | Delete Entity Files
*DatasetManagementApi* | [**get_dataset_entities_datasets_dataset_entities_user_id_user_type_id_dataset_id_get**](docs/DatasetManagementApi.md#get_dataset_entities_datasets_dataset_entities_user_id_user_type_id_dataset_id_get) | **GET** /datasets/datasetEntities/{user_id}/{user_type_id}/{dataset_id} | Get Dataset Entities
*DatasetManagementApi* | [**get_dataset_item_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_get**](docs/DatasetManagementApi.md#get_dataset_item_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_get) | **GET** /datasets/datasetEntities/{user_id}/{user_type_id}/{dataset_id}/{dataset_entity_id} | Get Dataset Item
*DatasetManagementApi* | [**get_entity_file_datasets_dataset_entities_files_user_id_user_type_id_dataset_id_file_id_get**](docs/DatasetManagementApi.md#get_entity_file_datasets_dataset_entities_files_user_id_user_type_id_dataset_id_file_id_get) | **GET** /datasets/datasetEntities/files/{user_id}/{user_type_id}/{dataset_id}/{file_id} | Get Entity File
*DatasetManagementApi* | [**get_thumbnail_datasets_dataset_entities_thumbnail_user_id_user_type_id_dataset_id_file_id_get**](docs/DatasetManagementApi.md#get_thumbnail_datasets_dataset_entities_thumbnail_user_id_user_type_id_dataset_id_file_id_get) | **GET** /datasets/datasetEntities/thumbnail/{user_id}/{user_type_id}/{dataset_id}/{file_id} | Get Thumbnail
*DatasetManagementApi* | [**get_user_datasets_datasets_user_id_user_type_id_get**](docs/DatasetManagementApi.md#get_user_datasets_datasets_user_id_user_type_id_get) | **GET** /datasets/{user_id}/{user_type_id} | Get User Datasets
*DatasetManagementApi* | [**new_dataset_datasets_user_id_user_type_id_post**](docs/DatasetManagementApi.md#new_dataset_datasets_user_id_user_type_id_post) | **POST** /datasets/{user_id}/{user_type_id} | New Dataset
*DatasetManagementApi* | [**new_dataset_entity_datasets_dataset_entities_user_id_user_type_id_post**](docs/DatasetManagementApi.md#new_dataset_entity_datasets_dataset_entities_user_id_user_type_id_post) | **POST** /datasets/datasetEntities/{user_id}/{user_type_id} | New Dataset Entity
*DatasetManagementApi* | [**recover_dataset_entities_datasets_dataset_entities_recover_user_id_user_type_id_post**](docs/DatasetManagementApi.md#recover_dataset_entities_datasets_dataset_entities_recover_user_id_user_type_id_post) | **POST** /datasets/datasetEntities/recover/{user_id}/{user_type_id} | Recover Dataset Entities
*DatasetManagementApi* | [**update_dataset_entity_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_put**](docs/DatasetManagementApi.md#update_dataset_entity_datasets_dataset_entities_user_id_user_type_id_dataset_id_dataset_entity_id_put) | **PUT** /datasets/datasetEntities/{user_id}/{user_type_id}/{dataset_id}/{dataset_entity_id} | Update Dataset Entity
*LocationManagementApi* | [**delete_location_locations_user_id_user_type_id_location_id_delete**](docs/LocationManagementApi.md#delete_location_locations_user_id_user_type_id_location_id_delete) | **DELETE** /locations/{user_id}/{user_type_id}/{location_id} | Delete Location
*LocationManagementApi* | [**get_user_locations_locations_user_id_user_type_id_get**](docs/LocationManagementApi.md#get_user_locations_locations_user_id_user_type_id_get) | **GET** /locations/{user_id}/{user_type_id} | Get User Locations
*LocationManagementApi* | [**new_location_locations_user_id_user_type_id_post**](docs/LocationManagementApi.md#new_location_locations_user_id_user_type_id_post) | **POST** /locations/{user_id}/{user_type_id} | New Location
*LocationManagementApi* | [**recover_location_locations_user_id_user_type_id_location_id_put**](docs/LocationManagementApi.md#recover_location_locations_user_id_user_type_id_location_id_put) | **PUT** /locations/{user_id}/{user_type_id}/{location_id} | Recover Location
*LocationManagementApi* | [**update_location_locations_user_id_user_type_id_put**](docs/LocationManagementApi.md#update_location_locations_user_id_user_type_id_put) | **PUT** /locations/{user_id}/{user_type_id} | Update Location
*SessionManagementApi* | [**delete_session_session_logout_user_id_user_type_id_delete**](docs/SessionManagementApi.md#delete_session_session_logout_user_id_user_type_id_delete) | **DELETE** /session/logout/{user_id}/{user_type_id} | Delete Session
*SessionManagementApi* | [**new_session_session_login_post**](docs/SessionManagementApi.md#new_session_session_login_post) | **POST** /session/login | New Session
*UsersManagementApi* | [**forgot_password_users_forgot_password_post**](docs/UsersManagementApi.md#forgot_password_users_forgot_password_post) | **POST** /users/forgotPassword | Forgot Password
*UsersManagementApi* | [**get_profile_picture_users_profile_pic_user_id_user_type_id_get**](docs/UsersManagementApi.md#get_profile_picture_users_profile_pic_user_id_user_type_id_get) | **GET** /users/profilePic/{user_id}/{user_type_id} | Get Profile Picture
*UsersManagementApi* | [**get_user_locations_users_students_user_id_user_type_id_get**](docs/UsersManagementApi.md#get_user_locations_users_students_user_id_user_type_id_get) | **GET** /users/students/{user_id}/{user_type_id} | Get User Locations
*UsersManagementApi* | [**new_student_users_students_user_id_user_type_id_post**](docs/UsersManagementApi.md#new_student_users_students_user_id_user_type_id_post) | **POST** /users/students/{user_id}/{user_type_id} | New Student
*UsersManagementApi* | [**reset_password_users_reset_password_post**](docs/UsersManagementApi.md#reset_password_users_reset_password_post) | **POST** /users/resetPassword | Reset Password
*UsersManagementApi* | [**update_profile_picture_users_profile_pic_user_id_user_type_id_post**](docs/UsersManagementApi.md#update_profile_picture_users_profile_pic_user_id_user_type_id_post) | **POST** /users/profilePic/{user_id}/{user_type_id} | Update Profile Picture
*UsersManagementApi* | [**update_user_profile_users_user_id_user_type_id_put**](docs/UsersManagementApi.md#update_user_profile_users_user_id_user_type_id_put) | **PUT** /users/{user_id}/{user_type_id} | Update User Profile
*UsersManagementApi* | [**validate_user_sign_up_users_validate_user_id_user_type_id_post**](docs/UsersManagementApi.md#validate_user_sign_up_users_validate_user_id_user_type_id_post) | **POST** /users/validate/{user_id}/{user_type_id} | Validate User Sign Up


## Documentation For Models

 - [DatasetDetails](docs/DatasetDetails.md)
 - [DatasetEntityDetails](docs/DatasetEntityDetails.md)
 - [DeleteDatasetEntities](docs/DeleteDatasetEntities.md)
 - [EntityFileDetails](docs/EntityFileDetails.md)
 - [ForgotPassword](docs/ForgotPassword.md)
 - [HTTPValidationError](docs/HTTPValidationError.md)
 - [LocationDetails](docs/LocationDetails.md)
 - [NewDataset](docs/NewDataset.md)
 - [NewDatasetEntity](docs/NewDatasetEntity.md)
 - [NewLocation](docs/NewLocation.md)
 - [NewSession](docs/NewSession.md)
 - [NewStudent](docs/NewStudent.md)
 - [ResetPassword](docs/ResetPassword.md)
 - [StudentDetails](docs/StudentDetails.md)
 - [UpdateDatasetEntity](docs/UpdateDatasetEntity.md)
 - [UpdateLocation](docs/UpdateLocation.md)
 - [UpdateUser](docs/UpdateUser.md)
 - [UserDetails](docs/UserDetails.md)
 - [ValidationError](docs/ValidationError.md)
 - [ValidationErrorLocInner](docs/ValidationErrorLocInner.md)


<a id="documentation-for-authorization"></a>
## Documentation For Authorization

Endpoints do not require authorization.


## Authors

Kumar R.,
Pavlovskiy E.N.,
Ivankov P.,
Denisov S.,
Mishchenko A.,
Bolotov K.

# Kappa - ϰ-framework for curating datasets and models
КАППА - Фреймворк курации датасетов и моделей
(Центр ИИ НГУ, Новосибирск)



## Назначение:

* Распределение ответственности при формировании набора данных;
* Контроль за обучением моделей машинного обучения с обратной связью на датасет;
* Испытание цифровых двойников на базе моделей искусственного интеллекта;
* Отслеживание соблюдения этических норм и стандартов


![](kappa-rus.png)

## Функции:

* Отслеживание авторства  разметки, в т.ч. использованием средств автоматизации разметки
* Индексация всех датасетов в интернете (для сферы строительства и городской среды)
* Индексация всех ИИ-задач, из научных публикаций и открытых кодов (для сферы строительства и городской среды)

# Проекты на базе фреймворка

* 05-2024 - 11-2024: [База данных](https://ai.nsu.ru/product/) для проекта ["Школьники - научные волонтёры"](https://syncwoia.com/event/datavolunteers)

## Финансовая поддержка

Исследование выполнено за счет финансовой поддержки (гранта) исследовательских центров,
предоставленной Автономной некоммерческой организацией «Аналитический центр при Правительстве
Российской Федерации», идентификатор соглашения о предоставлении субсидии 000000D730324P540002,
договор о предоставлении гранта с Новосибирским государственным университетом от 27.12.2023 № 70-2023-001318,
мероприятие № 23 «Выполнение работ в 2024 г. по ТЗ (проект Научные исследования в области ИИ (Фреймворк «КАППА»))»,
тематика Программы № 1 «Научные исследования в области ИИ для строительства и городской среды».

This work was supported by a grant for research centers, provided by the Analytical Center for
the Government of the Russian Federation in accordance with the subsidy agreement (agreement
identifier 000000D730324P540002) and the agreement with the Novosibirsk State University dated
December 27, 2023 No. 70-2023-001318.
