"""Soma API models."""

from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class MessageRole(str, Enum):
    """Message role enum."""

    USER = "user"
    AGENT = "agent"
    SYSTEM = "system"


class MessagePartTypeEnum(str, Enum):
    """Message part type enum."""

    TEXT_PART = "TextPart"
    FILE_PART = "FilePart"
    DATA_PART = "DataPart"


class TaskStatus(str, Enum):
    """Task status enum."""

    PENDING = "pending"
    WORKING = "working"
    INPUT_REQUIRED = "input-required"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


@dataclass
class MessagePart:
    """A part of a message."""

    type: MessagePartTypeEnum
    text: str | None = None
    file_url: str | None = None
    file_name: str | None = None
    mime_type: str | None = None
    data: Any | None = None
    metadata: dict[str, Any] = field(default_factory=dict)

    def model_dump(self) -> dict[str, Any]:
        """Convert to dictionary."""
        result: dict[str, Any] = {
            "type": self.type.value if isinstance(self.type, Enum) else self.type,
            "metadata": self.metadata,
        }
        if self.text is not None:
            result["text"] = self.text
        if self.file_url is not None:
            result["fileUrl"] = self.file_url
        if self.file_name is not None:
            result["fileName"] = self.file_name
        if self.mime_type is not None:
            result["mimeType"] = self.mime_type
        if self.data is not None:
            result["data"] = self.data
        return result


@dataclass
class Message:
    """A message in a task."""

    role: MessageRole
    parts: list[MessagePart]
    metadata: dict[str, Any] = field(default_factory=dict)
    reference_task_ids: list[str] = field(default_factory=list)

    def model_dump(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return {
            "role": self.role.value if isinstance(self.role, Enum) else self.role,
            "parts": [p.model_dump() for p in self.parts],
            "metadata": self.metadata,
            "referenceTaskIds": self.reference_task_ids,
        }


@dataclass
class CreateMessageRequest:
    """Request to create a message."""

    role: MessageRole | str
    parts: list[MessagePart | dict[str, Any]]
    metadata: dict[str, Any] = field(default_factory=dict)
    reference_task_ids: list[str] = field(default_factory=list)

    def model_dump(self) -> dict[str, Any]:
        """Convert to dictionary."""
        parts_data = []
        for p in self.parts:
            if isinstance(p, MessagePart):
                parts_data.append(p.model_dump())
            else:
                parts_data.append(p)

        return {
            "role": self.role.value if isinstance(self.role, Enum) else self.role,
            "parts": parts_data,
            "metadata": self.metadata,
            "referenceTaskIds": self.reference_task_ids,
        }


@dataclass
class CreateMessageResponse:
    """Response from creating a message."""

    id: str
    task_id: str
    message: Message | None = None


@dataclass
class TaskTimelineItem:
    """An item in the task timeline."""

    id: str
    type: str
    timestamp: str
    payload: dict[str, Any] = field(default_factory=dict)


@dataclass
class TaskTimelineResponse:
    """Response from task timeline endpoint."""

    items: list[TaskTimelineItem | dict[str, Any]]
    next_page_token: str | None = None
