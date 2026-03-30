"""Backlog / queue MCP tools."""

from __future__ import annotations

from typing import Any

from fastmcp import FastMCP

from agileplus_mcp.grpc_client import AgilePlusCoreClient


def register_tools(mcp: FastMCP, client: AgilePlusCoreClient) -> None:
    """Register backlog / queue tools onto *mcp*."""

    @mcp.tool(name="agileplus_queue_add")
    async def queue_add(
        title: str,
        description: str = "",
        item_type: str = "task",
        priority: str = "",
        source: str = "mcp",
        feature_slug: str = "",
        tags: list[str] | None = None,
    ) -> dict[str, Any]:
        item = await client.create_backlog_item(
            item_type=item_type,
            title=title,
            description=description,
            priority=priority,
            source=source,
            feature_slug=feature_slug,
            tags=tags,
        )
        return {"status": "success", "item": item}

    @mcp.tool(name="agileplus_queue_list")
    async def queue_list(
        item_type: str = "",
        status: str = "",
        priority: str = "",
        feature_slug: str = "",
        source: str = "",
        sort: str = "priority",
        limit: int = 20,
    ) -> dict[str, Any]:
        items = await client.list_backlog(
            type_filter=item_type or None,
            status_filter=status or None,
            priority_filter=priority or None,
            feature_slug=feature_slug or None,
            source_filter=source or None,
            sort=sort,
            limit=limit,
        )
        return {"status": "success", "items": items}

    @mcp.tool(name="agileplus_queue_show")
    async def queue_show(item_id: int) -> dict[str, Any]:
        item = await client.get_backlog_item(item_id)
        if item is None:
            return {"status": "not_found", "message": f"Backlog item {item_id} not found"}
        return {"status": "success", "item": item}

    @mcp.tool(name="agileplus_queue_pop")
    async def queue_pop(count: int = 1) -> dict[str, Any]:
        items = await client.pop_backlog_items(count=count)
        if not items:
            return {"status": "empty", "items": []}
        return {"status": "success", "items": items}

    @mcp.tool(name="agileplus_queue_import")
    async def queue_import(items: list[dict[str, Any]]) -> dict[str, Any]:
        """Import backlog items in a single batch."""
        for item in items:
            title = item.get("title")
            if not title:
                raise ValueError("Each imported backlog item requires a title")
        imported = await client.import_backlog_items(items)
        return {"status": "success", "imported": imported}
