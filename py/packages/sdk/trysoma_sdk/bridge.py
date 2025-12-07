"""Bridge client and function creation utilities."""

from dataclasses import dataclass
from typing import Any, Awaitable, Callable, Generic, TypeVar

try:
    from pydantic import BaseModel
except ImportError:
    BaseModel = object  # type: ignore

InputT = TypeVar("InputT", bound=BaseModel)
OutputT = TypeVar("OutputT", bound=BaseModel)


@dataclass
class ProviderController:
    """Provider controller configuration."""

    type_id: str
    name: str
    documentation: str = ""
    categories: list[str] | None = None
    credential_controllers: list[dict[str, Any]] | None = None

    def __post_init__(self) -> None:
        if self.categories is None:
            self.categories = []
        if self.credential_controllers is None:
            self.credential_controllers = []


@dataclass
class FunctionController:
    """Function controller configuration."""

    name: str
    description: str
    parameters: dict[str, Any] | None = None
    output: dict[str, Any] | None = None


@dataclass
class SomaFunction(Generic[InputT, OutputT]):
    """A Soma function with its metadata."""

    input_schema: type[InputT]
    output_schema: type[OutputT]
    provider_controller: ProviderController
    function_controller: FunctionController
    handler: Callable[[InputT], Awaitable[OutputT]]


def create_soma_function(
    *,
    input_schema: type[InputT],
    output_schema: type[OutputT],
    provider_controller: ProviderController,
    function_controller: FunctionController,
    handler: Callable[[InputT], Awaitable[OutputT]],
) -> SomaFunction[InputT, OutputT]:
    """Create a new Soma function.

    Args:
        input_schema: Pydantic model class for input validation.
        output_schema: Pydantic model class for output validation.
        provider_controller: Provider controller configuration.
        function_controller: Function controller configuration (name, description).
        handler: Async function that processes the input and returns the output.

    Returns:
        A SomaFunction instance.

    Example:
        ```python
        from pydantic import BaseModel

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
        )

        function = FunctionController(
            name="approve-claim",
            description="Approve a claim",
        )

        approve_claim = create_soma_function(
            input_schema=ClaimInput,
            output_schema=ClaimOutput,
            provider_controller=provider,
            function_controller=function,
            handler=lambda claim: ClaimOutput(approved=True),
        )
        ```
    """
    # Get JSON schema from pydantic models
    input_json_schema = None
    output_json_schema = None

    if hasattr(input_schema, "model_json_schema"):
        input_json_schema = input_schema.model_json_schema()
    if hasattr(output_schema, "model_json_schema"):
        output_json_schema = output_schema.model_json_schema()

    # Create the function controller with schemas
    full_function_controller = FunctionController(
        name=function_controller.name,
        description=function_controller.description,
        parameters=input_json_schema,
        output=output_json_schema,
    )

    return SomaFunction(
        input_schema=input_schema,
        output_schema=output_schema,
        provider_controller=provider_controller,
        function_controller=full_function_controller,
        handler=handler,
    )
