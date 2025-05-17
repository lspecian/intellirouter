from setuptools import setup, find_packages

with open("intellirouter/README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="intellirouter",
    version="0.1.0",
    description="Python SDK for IntelliRouter",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="IntelliRouter Team",
    author_email="info@intellirouter.com",
    url="https://github.com/intellirouter/intellirouter-python",
    packages=find_packages(),
    install_requires=[
        "requests>=2.25.0",
        "aiohttp>=3.7.4",
        "sseclient-py>=1.7.2",
        "pydantic>=1.8.0",
    ],
    extras_require={
        "dev": [
            "pytest>=6.0.0",
            "pytest-asyncio>=0.14.0",
            "pytest-cov>=2.10.0",
            "black>=20.8b1",
            "isort>=5.7.0",
            "mypy>=0.800",
            "sphinx>=3.5.0",
            "sphinx-rtd-theme>=0.5.1",
        ],
    },
    python_requires=">=3.7",
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
)