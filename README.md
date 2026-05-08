# kf_sdk

[![License BSD 3.0](https://img.shields.io/github/license/nsu-ai/kappa.svg)](https://github.com/nsu-ai/kappa/blob/master/LICENSE.md)
![Python 3.8, 3.10](https://img.shields.io/badge/python-3.8%20%7C%203.10-green.svg)
[![PyPI Downloads](https://static.pepy.tech/badge/kf-sdk)](https://pepy.tech/projects/kf-sdk)
![Releases](https://img.shields.io/github/release/nsu-ai/kappa.svg)

Kappa Framework Python SDK for Kappa v.1.0.0

- API version: 0.1.0
- Package version: 1.0.0

# Каппа - ϰ-фреймворк управления датасетами, версия 2.0.0
Каппа - набор концептуального и программного обеспечения (фреймворк) для осуществления функций курации датасетов и моделей
([Исследовательский центр](https://nsu.ru/n/ai-center) в сфере искусственного интеллекта по направлению "Строительство и городская среда" НГУ, Новосибирск)

## Авторы

Кумар Р., Павловский Е.Н., Иванков П.С., Денисов С.С., Мищенко А.С., Болотов К.Ю., Безруков Я.С., Глушенко А.В., Дербаль Р., Бобо С.


## Назначение:

* Распределение ответственности при формировании набора данных;
* Контроль за обучением моделей машинного обучения с обратной связью на датасет;
* Испытание цифровых двойников на базе моделей искусственного интеллекта;
* Отслеживание соблюдения этических норм и стандартов


![](kappa-rus.png)

## Функции:

* [Реализовано в 2024, версия 1.0.0] Отслеживание авторства разметки, в т.ч. использованием средств автоматизации разметки
* [Реализовано в 2025, версия 2.0.0] Бенчмаркинг ИИ-моделей
* [План на 2026] Индексация всех датасетов в интернете (для сферы строительства и городской среды), индексация всех ИИ-задач, из научных публикаций и открытых кодов (для сферы строительства и городской среды)

## Проекты на базе фреймворка

* 05-2024 - 11-2024: [База данных](https://ai.nsu.ru/dv/) для проекта ["Школьники - научные волонтёры"](https://syncwoia.com/event/datavolunteers)
* 12-2024 - н.в.: [Развёрнутая версия фреймворка с датасетами](https://kappa.nsu.ru:8060/user-micro-services/v1/docs), здесь можно скачать
[датасет библиографических карточек](#Датасеты) по теме строительства, который используется для тренировки алгоритмов распознавания текста на изображениях.

## Финансовая поддержка

2024: Исследование выполнено за счет финансовой поддержки (гранта) исследовательских центров,
предоставленной Автономной некоммерческой организацией «Аналитический центр при Правительстве
Российской Федерации», идентификатор соглашения о предоставлении субсидии 000000D730324P540002,
договор о предоставлении гранта с Новосибирским государственным университетом от 27.12.2023 № 70-2023-001318,
мероприятие № 23 «Выполнение работ в 2024 г. по ТЗ (проект Научные исследования в области ИИ (Фреймворк «КАППА»))»,
тематика Программы № 1 «Научные исследования в области ИИ для строительства и городской среды».

2024: This work was supported by a grant for research centers, provided by the Analytical Center for
the Government of the Russian Federation in accordance with the subsidy agreement (agreement
identifier 000000D730324P540002) and the agreement with the Novosibirsk State University dated
December 27, 2023 No. 70-2023-001318.

2025: Исследование выполнено в рамках
выполнения гранта (Соглашение о предоставлении из федерального бюджета гранта в форме субсидии от
17.04.2025 № 139-15-2025-006 ИГК 000000Ц313925P3S0002):
- относится к направлению развития искусственного интеллекта: «Строительство и городская среда»;
- относится к мероприятию Плана деятельности Исследовательского центра в сфере искусственного
интеллекта Новосибирского государственного университета № 56 «Выполнение работ в 2025 г. по ТЗ
(проект Научные исследования в области ИИ (Фреймворк «КАППА»))»;
- относится к тематике Программы Исследовательского центра в сфере искусственного интеллекта
Новосибирского государственного университета: «Тематика 1. Научные исследования в области ИИ для
Строительства и городской среды».

2026: Исследование выполнено в рамках выполнения гранта (Соглашение о предоставлении из федерального бюджета
гранта в форме субсидии от 17.04.2025 № 139-15-2025-006 ИГК 000000Ц313925P3S0002):
- относится к направлению развития искусственного интеллекта: «Строительство и городская среда»;
- относится к мероприятию Плана деятельности Исследовательского центра в сфере искусственного
интеллекта Новосибирского государственного университета № 85 «Выполнение работ в 2026 г. по ТЗ
(проект Научные исследования в области ИИ (Фреймворк «КАППА»))»;
- относится к тематике Программы Исследовательского центра в сфере искусственного интеллекта
Новосибирского государственного университета: «Тематика 1. Научные исследования в области ИИ для
Строительства и городской среды».


## Датасеты

На базе фреймворка зарегистрировано три датасета:
* "Датасет по ретроконверсии библиотечных карточек (строительство)", свидетельство о регистрации базы данных №2025620303 от 17 января 2025 г.
* "Датасет по наблюдениям за флорой и фауной в городской среде",свидетельство о регистрации базы данных № 2025620139 от 10 января 2025 г.
* "Датасет по извлечению аннотаций из библиотечных ресурсов", свидетельство о регистрации базы данных №2026620302 от 21 января 2026 г.
 
# kf_sdk installation

## Requirements

Python 3.8+

Libraries:
```
urllib3 >= 1.25.3, < 3.0.0
python_dateutil >= 2.8.2
pydantic >= 2
typing-extensions >= 4.7.1
```

## Installation & Usage
### pip install

If the python package is hosted on PyPI, you can install directly using:

```sh
pip install kf_sdk
```

Then import the package:
```python
import kf_sdk
```

### Setuptools

Install via [Setuptools](http://pypi.python.org/pypi/setuptools).

```sh
git clone https://github.com/nsu-ai/kappa.git
cd kappa
python setup.py install --user
```
(or `sudo python setup.py install` to install the package for all users)

Then import the package:
```python
import kf_sdk
```

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
    host = "https://kappa.nsu.ru:8060/user-micro-services/v1"
)

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
        print("The response of SessionManagementApi->get_new_session:\n")
        pprint(api_response)
    except ApiException as e:
        print("Exception when calling SessionManagementApi->get_new_session: %s\n" % e)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure Bearer authorization: HTTPBearer
configuration_data = kf_sdk.Configuration(
    host = "https://kappa.nsu.ru:8060/data-micro-services/v1",
    access_token = token
)


with kf_sdk.ApiClient(configuration_data) as api_client:
    # Create an instance of the API class
    api_instance = kf_sdk.DatasetManagementApi(api_client)
    dataset_id = 582
    dataset_version_no = '1.0.0'

    try:
        # Get Zip file of the Dataset  under Version
        api_response = api_instance.get_dataset_version_archive_datasets_versions_archive_user_id_user_type_id_dataset_id_dataset_version_no_get_without_preload_content(user_id, user_type_id, dataset_id, dataset_version_no)
        pprint(api_response)
        with open("tmp.zip","wb") as f:
            f.write(api_response.data)
    except Exception as e:
        print("Exception when calling DatasetManagementApi->get_dataset_version_datasets_versions_user_id_user_type_id_dataset_id_dataset_version_no_get_without_preload_content: %s\n" % e)

```

## Documentation for API Endpoints

All URIs are relative to *http://localhost*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*DatasetManagementApi* | [**get_dataset_version_archive_datasets_versions_archive_user_id_user_type_id_dataset_id_dataset_version_no_get**](docs/DatasetManagementApi.md#get_dataset_version_archive_datasets_versions_archive_user_id_user_type_id_dataset_id_dataset_version_no_get) | **GET** /datasets/datasetEntities/{user_id}/{user_type_id} | Get Dataset Version
*SessionManagementApi* | [**delete_session_session_logout_user_id_user_type_id_delete**](docs/SessionManagementApi.md#delete_session_session_logout_user_id_user_type_id_delete) | **DELETE** /session/logout/{user_id}/{user_type_id} | Delete Session
*SessionManagementApi* | [**get_new_session**](docs/SessionManagementApi.md#get_new_session) | **POST** /session/login | New Session


## Documentation For Models

 - [DatasetDetails](docs/DatasetDetails.md)  
 - [HTTPValidationError](docs/HTTPValidationError.md)    
 - [NewSession](docs/NewSession.md)      
 - [ValidationError](docs/ValidationError.md)
 - [ValidationErrorLocInner](docs/ValidationErrorLocInner.md)


<a id="documentation-for-authorization"></a>
## Documentation For Authorization

Endpoints do not require authorization.



