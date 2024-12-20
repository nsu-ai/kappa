# coding: utf-8

"""
    Kappa-Framework SDK based on FastAPI

    Kappa Framework for Datasets Management from NSU-AI Center

"""  # noqa: E501


from setuptools import setup, find_packages  # noqa: H301

# To install the library, run the following
#
# python setup.py install
#
# prerequisite: setuptools
# http://pypi.python.org/pypi/setuptools
NAME = "kf_sdk"
VERSION = "1.0.0"
PYTHON_REQUIRES = ">= 3.8"
REQUIRES = [
    "urllib3 >= 1.25.3, < 3.0.0",
    "python-dateutil >= 2.8.2",
    "pydantic >= 2",
    "typing-extensions >= 4.7.1",
]

setup(
    name=NAME,
    version=VERSION,
    description="Kappa Framework for Datasets Management from NSU-AI Center",
    author="Kappa group from A.I. Center of NSU",
    author_email="ai@nsu.ru",
    url="https://github.com/nsu-ai/kappa",
    keywords=["Kappa Framework", "Datasets Management", "FastAPI"],
    install_requires=REQUIRES,
    license="BSD-3.0 License",
    packages=find_packages(exclude=["test", "tests"]),
    include_package_data=True,
    long_description_content_type='text/markdown',
    long_description="""\
    Kappa Framwork Python SDK for Kappa v.1.0.0
    """,  # noqa: E501
    package_data={"kf_sdk": ["py.typed"]},
)
