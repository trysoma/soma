"""Bridge client and function creation utilities."""

from dataclasses import dataclass
from typing import Awaitable, Callable, Generic, TypeVar

from pydantic import BaseModel

from trysoma_sdk_core import (
    FunctionMetadata,
    ProviderController,
)

InputT = TypeVar("InputT", bound=BaseModel)
OutputT = TypeVar("OutputT", bound=BaseModel)


@dataclass
class SomaFunction(Generic[InputT, OutputT]):
    """A Soma function with its metadata."""

    input_schema: type[InputT]
    output_schema: type[OutputT]
    provider_controller: ProviderController
    function_metadata: FunctionMetadata
    handler: Callable[[InputT], Awaitable[OutputT]]


def create_soma_function(
    *,
    input_schema: type[InputT],
    output_schema: type[OutputT],
    provider_controller: ProviderController,
    function_name: str,
    function_description: str,
    handler: Callable[[InputT], Awaitable[OutputT]],
) -> SomaFunction[InputT, OutputT]:
    """Create a new Soma function.

    Args:
        input_schema: Pydantic model class for input validation.
        output_schema: Pydantic model class for output validation.
        provider_controller: Provider controller from trysoma_sdk_core.
        function_name: Name of the function.
        function_description: Description of what the function does.
        handler: Async function that processes the input and returns the output.

    Returns:
        A SomaFunction instance.

    Example:
        ```python
        from pydantic import BaseModel
        from trysoma_sdk import (
            create_soma_function,
            ProviderController,
            ProviderCredentialController,
        )

        class ClaimInput(BaseModel):
            date: str
            category: str
            reason: str
            amount: float
            email: str

        class ClaimOutput(BaseModel):
            approved: bool

        provider = ProviderController(
            type_id="approve-claim",
            name="Approve Claim",
            documentation="Approve a claim",
            categories=["insurance"],
            credential_controllers=[ProviderCredentialController.no_auth()],
        )

        async def handle_claim(claim: ClaimInput) -> ClaimOutput:
            return ClaimOutput(approved=True)

        approve_claim = create_soma_function(
            input_schema=ClaimInput,
            output_schema=ClaimOutput,
            provider_controller=provider,
            function_name="approve-claim",
            function_description="Approve a claim",
            handler=handle_claim,
        )
        ```
    """
    import json

    # Get JSON schema from pydantic models
    input_json_schema = input_schema.model_json_schema()
    output_json_schema = output_schema.model_json_schema()

    # Create the function metadata with schemas as JSON strings
    function_metadata = FunctionMetadata(
        function_name,
        function_description,
        json.dumps(input_json_schema),
        json.dumps(output_json_schema),
    )

    return SomaFunction(
        input_schema=input_schema,
        output_schema=output_schema,
        provider_controller=provider_controller,
        function_metadata=function_metadata,
        handler=handler,
    )
