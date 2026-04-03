"""MCP feature listing integration tests."""

from __future__ import annotations

import pytest


@pytest.mark.asyncio
async def test_list_features(client):
    """list_features should return a list of feature dicts."""
    features = await client.list_features()
    assert isinstance(features, list), "Expected a list of features"

    required = ["id", "slug", "state", "friendly_name"]
    for feature in features:
        for field in required:
            assert field in feature, f"Feature missing field '{field}': {feature}"
