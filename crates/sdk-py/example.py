"""Example usage of the Python SDK"""
import asyncio
import json
from sdk_py import register_function_handler, start_sdk_server, InvocationResponse

# Define your providers
providers = [
    {
        "type_id": "my_provider",
        "name": "My Provider",
        "documentation": "Example provider",
        "categories": ["example"],
        "functions": [
            {
                "name": "my_function",
                "description": "Example function",
                "parameters": "{}",
                "output": "{}"
            }
        ],
        "credential_controllers": []
    }
]

# Define a handler function
def my_function_handler(request):
    """Handle function invocations

    Args:
        request: InvocationRequest with:
            - provider_controller_type_id
            - function_controller_type_id
            - credential_controller_type_id
            - credentials (JSON string)
            - parameters (JSON string)

    Returns:
        InvocationResponse with success, data, and error fields
    """
    print(f"Function invoked: {request.provider_controller_type_id}:{request.function_controller_type_id}")

    try:
        # Parse the parameters
        params = json.loads(request.parameters)

        # Do your logic here
        result = {"message": "Hello from Python!"}

        # Return success response
        response = InvocationResponse()
        response.success = True
        response.data = json.dumps(result)
        response.error = None
        return response
    except Exception as e:
        # Return error response
        response = InvocationResponse()
        response.success = False
        response.data = None
        response.error = str(e)
        return response

# Register the handler
register_function_handler(
    "my_provider",
    "my_function",
    my_function_handler
)

# Start the server
def main():
    try:
        start_sdk_server(
            json.dumps(providers),
            "/tmp/soma-sdk.sock"
        )
    except Exception as error:
        print(f"Failed to start server: {error}")

if __name__ == "__main__":
    main()
