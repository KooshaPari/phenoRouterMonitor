"""Phenotype Core: Foundation package for config, errors, logging, and observability."""

from . import config, errors, logging, observability

__version__ = "0.1.0"
__author__ = "Phenotype Team"
__all__ = [
    "errors",
    "config",
    "logging",
    "observability",
]
