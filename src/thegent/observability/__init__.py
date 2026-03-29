"""Observability module - telemetry, metrics, and event egress."""

from .egress import EgressEvent, SIEMEgress

__all__ = ["EgressEvent", "SIEMEgress"]
