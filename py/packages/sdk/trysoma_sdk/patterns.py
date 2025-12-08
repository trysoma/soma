"""Agent interaction patterns for Soma SDK."""

from dataclasses import dataclass
from typing import Awaitable, Callable, Generic, TypeVar
from uuid import UUID

from restate import ObjectContext
from restate.context import RestateDurableFuture
from trysoma_api_client import V1Api as SomaV1Api
from trysoma_api_client.models.create_message_request import CreateMessageRequest
from trysoma_api_client.models.create_message_response import CreateMessageResponse
from trysoma_api_client.models.task_timeline_item import TaskTimelineItem
from trysoma_api_client.models.task_timeline_item_paginated_response import (
    TaskTimelineItemPaginatedResponse,
)


BridgeT = TypeVar("BridgeT")
InputT = TypeVar("InputT")
OutputT = TypeVar("OutputT")

FirstTurn = str  # Literal["user", "agent"]


@dataclass
class ChatHandlerParams(Generic[BridgeT, InputT, OutputT]):
    """Parameters passed to chat pattern handlers."""

    ctx: ObjectContext
    soma: SomaV1Api
    bridge: BridgeT
    history: list[TaskTimelineItem]
    input: InputT
    on_goal_achieved: Callable[[OutputT], None]
    send_message: Callable[[CreateMessageRequest], Awaitable[CreateMessageResponse]]


@dataclass
class WrappedChatHandlerParams(Generic[BridgeT, InputT, OutputT]):
    """Parameters for wrapped chat handler."""

    ctx: ObjectContext
    soma: SomaV1Api
    bridge: BridgeT
    input: InputT
    task_id: str
    first_turn: FirstTurn = "user"


@dataclass
class WorkflowHandlerParams(Generic[BridgeT, InputT, OutputT]):
    """Parameters passed to workflow pattern handlers."""

    ctx: ObjectContext
    soma: SomaV1Api
    bridge: BridgeT
    history: list[TaskTimelineItem]
    input: InputT
    send_message: Callable[[CreateMessageRequest], Awaitable[CreateMessageResponse]]
    interruptable: bool


@dataclass
class WrappedWorkflowHandlerParams(Generic[BridgeT, InputT, OutputT]):
    """Parameters for wrapped workflow handler."""

    ctx: ObjectContext
    soma: SomaV1Api
    bridge: BridgeT
    input: InputT
    task_id: str
    interruptable: bool = True


def chat(
    handler: Callable[[ChatHandlerParams[BridgeT, InputT, OutputT]], Awaitable[None]],
) -> Callable[[WrappedChatHandlerParams[BridgeT, InputT, OutputT]], Awaitable[OutputT]]:
    """Create a chat pattern handler.

    The chat pattern is used for conversational agents that need to:
    - Wait for user input
    - Process messages in a loop
    - Exit when a goal is achieved

    Args:
        handler: The handler function that processes each turn of the conversation.

    Returns:
        A wrapped handler that manages the chat loop.

    Example:
        ```python
        async def discover_claim(params: ChatHandlerParams) -> None:
            # Process the conversation
            # Call params.on_goal_achieved(result) when done
            pass

        discover = patterns.chat(discover_claim)

        # In agent entrypoint:
        result = await discover(WrappedChatHandlerParams(
            ctx=ctx,
            soma=soma,
            bridge=bridge,
            input={"model": model},
            task_id=task_id,
            first_turn="agent",
        ))
        ```
    """

    async def wrapped(
        params: WrappedChatHandlerParams[BridgeT, InputT, OutputT],
    ) -> OutputT:
        NEW_INPUT_PROMISE = "new_input_promise"
        ctx = params.ctx
        soma = params.soma

        # Create awakeable for waiting for new input
        awakeable_id: str
        new_input_promise: RestateDurableFuture[dict[str, str]]
        awakeable_id, new_input_promise = ctx.awakeable()
        ctx.set(NEW_INPUT_PROMISE, awakeable_id)

        goal_output: OutputT | None = None
        achieved = False

        def on_goal_achieved(output: OutputT) -> None:
            nonlocal goal_output, achieved
            goal_output = output
            achieved = True

        if params.first_turn == "user":
            await new_input_promise

        while not achieved:
            # Fetch message history
            async def fetch_history() -> TaskTimelineItemPaginatedResponse:
                return soma.task_history(page_size=1000, task_id=UUID(params.task_id))

            messages: TaskTimelineItemPaginatedResponse = await ctx.run_typed(
                "fetch_history",
                fetch_history,
            )

            async def send_message(
                message: CreateMessageRequest,
            ) -> CreateMessageResponse:
                async def send() -> CreateMessageResponse:
                    return soma.send_message(
                        task_id=UUID(params.task_id),
                        create_message_request=message,
                    )

                return await ctx.run_typed("send_message", send)

            handler_params: ChatHandlerParams[BridgeT, InputT, OutputT] = (
                ChatHandlerParams(
                    ctx=ctx,
                    soma=soma,
                    history=messages.items,
                    bridge=params.bridge,
                    input=params.input,
                    on_goal_achieved=on_goal_achieved,
                    send_message=send_message,
                )
            )

            await handler(handler_params)

            if not achieved:
                # Re-arm the awakeable, waiting for another message
                new_id: str
                next_promise: RestateDurableFuture[dict[str, str]]
                new_id, next_promise = ctx.awakeable()
                ctx.set(NEW_INPUT_PROMISE, new_id)
                await next_promise

        if goal_output is None:
            raise RuntimeError("Goal not achieved")

        return goal_output

    return wrapped


def workflow(
    handler: Callable[
        [WorkflowHandlerParams[BridgeT, InputT, OutputT]], Awaitable[OutputT]
    ],
) -> Callable[
    [WrappedWorkflowHandlerParams[BridgeT, InputT, OutputT]], Awaitable[OutputT]
]:
    """Create a workflow pattern handler.

    The workflow pattern is used for agents that need to:
    - Execute a sequence of steps
    - Optionally be interrupted by new input
    - Return a result when complete

    Args:
        handler: The handler function that executes the workflow.

    Returns:
        A wrapped handler that manages the workflow execution.

    Example:
        ```python
        async def process_claim(params: WorkflowHandlerParams) -> None:
            # Process the claim
            await params.send_message({...})
            pass

        process = patterns.workflow(process_claim)

        # In agent entrypoint:
        await process(WrappedWorkflowHandlerParams(
            ctx=ctx,
            soma=soma,
            bridge=bridge,
            input={"assessment": assessment},
            task_id=task_id,
            interruptable=False,
        ))
        ```
    """

    async def wrapped(
        params: WrappedWorkflowHandlerParams[BridgeT, InputT, OutputT],
    ) -> OutputT:
        import asyncio

        NEW_INPUT_PROMISE = "new_input_promise"
        ctx = params.ctx
        soma = params.soma

        while True:
            # Create awakeable for waiting for new input
            awakeable_id: str
            new_input_promise: RestateDurableFuture[dict[str, str]]
            awakeable_id, new_input_promise = ctx.awakeable()
            ctx.set(NEW_INPUT_PROMISE, awakeable_id)

            # Fetch message history
            async def fetch_history() -> TaskTimelineItemPaginatedResponse:
                return soma.task_history(page_size=1000, task_id=UUID(params.task_id))

            messages: TaskTimelineItemPaginatedResponse = await ctx.run_typed(
                "fetch_history",
                fetch_history,
            )

            async def send_message(
                message: CreateMessageRequest,
            ) -> CreateMessageResponse:
                async def send() -> CreateMessageResponse:
                    return soma.send_message(
                        task_id=UUID(params.task_id),
                        create_message_request=message,
                    )

                return await ctx.run_typed("send_message", send)

            handler_params: WorkflowHandlerParams[BridgeT, InputT, OutputT] = (
                WorkflowHandlerParams(
                    ctx=ctx,
                    soma=soma,
                    history=messages.items,
                    bridge=params.bridge,
                    input=params.input,
                    send_message=send_message,
                    interruptable=params.interruptable,
                )
            )

            if params.interruptable:
                # Race between new input and handler completion
                async def await_handler() -> OutputT:
                    return await handler(handler_params)

                async def await_input() -> dict[str, str]:
                    return await new_input_promise

                handler_task: asyncio.Task[OutputT] = asyncio.create_task(
                    await_handler()
                )
                input_task: asyncio.Task[dict[str, str]] = asyncio.create_task(
                    await_input()
                )

                done, pending = await asyncio.wait(
                    [handler_task, input_task],
                    return_when=asyncio.FIRST_COMPLETED,
                )

                for task in pending:
                    task.cancel()

                if handler_task in done:
                    result: OutputT = handler_task.result()
                    return result
                # Otherwise loop to handle new input
            else:
                # Not interruptable, just wait for handler to complete
                return await handler(handler_params)

        # This should never be reached, but makes type checker happy
        raise RuntimeError("Workflow loop exited unexpectedly")

    return wrapped


class Patterns:
    """Container for pattern functions."""

    chat = staticmethod(chat)
    workflow = staticmethod(workflow)


patterns = Patterns()
