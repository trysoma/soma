"""Utility functions for the insurance claim bot."""

from typing import Any

from trysoma_api_client.models.task_timeline_item import TaskTimelineItem


def convert_to_openai_messages(history: list[TaskTimelineItem]) -> list[dict[str, Any]]:
    """Convert Soma task history to OpenAI message format.

    Args:
        history: List of TaskTimelineItem from Soma API.

    Returns:
        List of messages in OpenAI format.
    """
    messages: list[dict[str, Any]] = []

    # Sort by created_at
    sorted_history = sorted(history, key=lambda x: x.created_at)

    for item in sorted_history:
        # Get the actual payload instance
        payload = item.event_payload.actual_instance
        if payload is None:
            continue

        # Check if this is a message type (not a status update)
        if not hasattr(payload, "type") or payload.type != "message":
            continue

        # Get the message
        if not hasattr(payload, "message"):
            continue

        message = payload.message

        # Get role and convert to OpenAI format
        role = message.role if hasattr(message, "role") else "user"
        openai_role = "assistant" if role == "agent" else "user"

        # Extract text content from parts
        parts = message.parts if hasattr(message, "parts") else []
        content_parts: list[str] = []
        for part in parts:
            # Handle both text-part and TextPart enum values
            part_type = part.type if hasattr(part, "type") else None
            if part_type in ("text-part", "TextPart") and hasattr(part, "text"):
                if part.text:
                    content_parts.append(part.text)

        if content_parts:
            messages.append({
                "role": openai_role,
                "content": "\n".join(content_parts),
            })

    return messages
