"""Backlog RPC helpers for the AgilePlus gRPC client."""

from __future__ import annotations

from typing import Any

from agileplus_mcp.grpc_errors import GrpcCallError


class AgilePlusBacklogGrpcMixin:
    """Backlog-related RPC helpers for `AgilePlusCoreClient`."""

    async def create_backlog_item(
        self,
        item_type: str,
        title: str,
        description: str = "",
        priority: str = "",
        source: str = "mcp",
        feature_slug: str = "",
        tags: list[str] | None = None,
    ) -> dict[str, Any]:
        """Create a backlog item via the integrations service."""
        from agileplus_proto.gen.agileplus.v1 import integrations_pb2  # type: ignore[import]

        stub = self._require_integrations_stub()
        request = integrations_pb2.CreateBacklogItemRequest(
            type=item_type,
            title=title,
            description=description,
            priority=priority,
            source=source,
            feature_slug=feature_slug,
            tags=list(tags or []),
        )
        response = await self._call_with_retry(lambda: stub.CreateBacklogItem(request))
        return self._backlog_item_to_dict(response.item)

    async def list_backlog(
        self,
        type_filter: str | None = None,
        status_filter: str | None = None,
        priority_filter: str | None = None,
        feature_slug: str | None = None,
        source_filter: str | None = None,
        sort: str = "",
        limit: int = 0,
    ) -> list[dict[str, Any]]:
        """List backlog items via the integrations service."""
        from agileplus_proto.gen.agileplus.v1 import integrations_pb2  # type: ignore[import]

        stub = self._require_integrations_stub()
        request = integrations_pb2.ListBacklogRequest(
            type_filter=type_filter or "",
            state_filter=status_filter or "",
            feature_slug=feature_slug or "",
            priority_filter=priority_filter or "",
            source_filter=source_filter or "",
            sort=sort,
            limit=limit,
        )
        response = await self._call_with_retry(lambda: stub.ListBacklog(request))
        return [self._backlog_item_to_dict(item) for item in response.items]

    async def get_backlog_item(self, backlog_item_id: int) -> dict[str, Any] | None:
        from agileplus_proto.gen.agileplus.v1 import integrations_pb2  # type: ignore[import]

        stub = self._require_integrations_stub()
        request = integrations_pb2.GetBacklogItemRequest(
            backlog_item_id=backlog_item_id,
        )
        try:
            response = await self._call_with_retry(lambda: stub.GetBacklogItem(request))
        except GrpcCallError as exc:
            if getattr(exc.code, "name", "") == "NOT_FOUND":
                return None
            raise
        return self._backlog_item_to_dict(response.item)

    async def import_backlog_items(self, items: list[dict[str, Any]]) -> list[dict[str, Any]]:
        """Import backlog items in a single batch."""
        from agileplus_proto.gen.agileplus.v1 import integrations_pb2  # type: ignore[import]

        stub = self._require_integrations_stub()
        request_items = []
        for item in items:
            title = str(item.get("title") or "").strip()
            if not title:
                raise ValueError("Each imported backlog item requires a title")

            request_items.append(
                integrations_pb2.CreateBacklogItemRequest(
                    type=str(item.get("type") or item.get("item_type") or "task"),
                    title=title,
                    description=str(item.get("description") or item.get("body") or ""),
                    priority=str(item.get("priority") or ""),
                    source=str(item.get("source") or "mcp"),
                    feature_slug=str(item.get("feature_slug") or ""),
                    tags=list(item.get("tags") or []),
                )
            )

        request = integrations_pb2.ImportBacklogRequest(items=request_items)
        response = await self._call_with_retry(lambda: stub.ImportBacklog(request))
        return [self._backlog_item_to_dict(item) for item in response.items]

    async def update_backlog_status(
        self, backlog_item_id: int, target_status: str
    ) -> dict[str, Any]:
        """Update the backlog item's lifecycle status."""
        from agileplus_proto.gen.agileplus.v1 import integrations_pb2  # type: ignore[import]

        stub = self._require_integrations_stub()
        request = integrations_pb2.UpdateBacklogStatusRequest(
            backlog_item_id=backlog_item_id,
            target_status=target_status,
        )
        response = await self._call_with_retry(lambda: stub.UpdateBacklogStatus(request))
        return {
            "backlog_item_id": response.backlog_item_id,
            "from_status": response.from_status,
            "to_status": response.to_status,
        }

    async def pop_backlog_items(self, count: int = 1) -> list[dict[str, Any]]:
        """Pop the next backlog items in priority order."""
        from agileplus_proto.gen.agileplus.v1 import integrations_pb2  # type: ignore[import]

        stub = self._require_integrations_stub()
        request = integrations_pb2.PopBacklogRequest(count=count)
        response = await self._call_with_retry(lambda: stub.PopBacklog(request))
        return [self._backlog_item_to_dict(item) for item in response.items]

    @staticmethod
    def _backlog_item_to_dict(item: Any) -> dict[str, Any]:
        return {
            "id": item.id,
            "type": item.type,
            "title": item.title,
            "description": item.description,
            "priority": item.priority,
            "state": item.state,
            "source": item.source,
            "feature_slug": item.feature_slug,
            "tags": list(item.tags),
            "created_at": item.created_at,
            "updated_at": item.updated_at,
        }
