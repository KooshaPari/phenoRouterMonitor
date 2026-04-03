from setuptools import setup, find_packages, Extension
from setuptools_rust import RustExtension

setup(
    name="phenotype-skills",
    version="0.1.0",
    author="Phenotype Contributors",
    description="Modular Skill System for Agent Orchestration",
    long_description=open("README.md").read(),
    long_description_content_type="text/markdown",
    url="https://github.com/phenotype/skills",
    license="MIT",
    packages=find_packages(),
    rust_extensions=[
        RustExtension(
            "phenotype_skills._core",
            path="../../Cargo.toml",
            binding="pyo3",
        )
    ],
    install_requires=[],
    python_requires=">=3.8",
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Rust",
    ],
    zip_safe=False,
)
