"""Utility functions for the insurance claim bot."""

from typing import Any


def convert_to_openai_messages(history: list[Any]) -> list[dict[str, Any]]:
    """Convert Soma task history to OpenAI message format.

    Args:
        history: List of TaskTimelineItem from Soma API.

    Returns:
        List of messages in OpenAI format.
    """
    messages: list[dict[str, Any]] = []

    for item in history:
        # Handle both dict and object access
        if isinstance(item, dict):
            payload = item.get("payload", {})
            item_type = item.get("type", "")
        else:
            payload = getattr(item, "payload", {})
            item_type = getattr(item, "type", "")

        if item_type != "message":
            continue

        # Extract message from payload
        if isinstance(payload, dict):
            message = payload.get("message", payload)
        else:
            message = payload

        # Get role and parts
        if isinstance(message, dict):
            role = message.get("role", "user")
            parts = message.get("parts", [])
        else:
            role = getattr(message, "role", "user")
            parts = getattr(message, "parts", [])

        # Convert role
        openai_role = "assistant" if role == "agent" else role

        # Extract text content from parts
        content_parts: list[str] = []
        for part in parts:
            if isinstance(part, dict):
                if part.get("type") == "TextPart" and part.get("text"):
                    content_parts.append(part["text"])
            else:
                if getattr(part, "type", None) == "TextPart":
                    text = getattr(part, "text", None)
                    if text:
                        content_parts.append(text)

        if content_parts:
            messages.append({
                "role": openai_role,
                "content": "\n".join(content_parts),
            })

    return messages
