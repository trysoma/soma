"""Standalone server generation for Soma Python SDK.

This module is the Python equivalent of vite.ts in the JS SDK.
It generates a standalone.py file that runs the SDK server.
"""

import os
import sys
from pathlib import Path
from typing import Any


def find_python_files(directory: Path, base_dir: Path | None = None) -> dict[str, Path]:
    """Recursively find all .py files in a directory.

    Args:
        directory: The directory to search.
        base_dir: The base directory for relative paths.

    Returns:
        A dict mapping entry names to file paths.
    """
    if base_dir is None:
        base_dir = directory

    entries: dict[str, Path] = {}

    if not directory.exists():
        return entries

    for file in directory.iterdir():
        if file.is_dir():
            entries.update(find_python_files(file, base_dir))
        elif file.suffix == ".py" and not file.name.startswith("_"):
            relative_path = file.relative_to(base_dir)
            # Create entry name by removing .py extension
            entry_name = str(relative_path.with_suffix("")).replace(os.sep, "/")
            entries[entry_name] = file

    return entries


def generate_standalone_server(base_dir: Path, is_dev: bool = False) -> str:
    """Generate the standalone server Python code.

    Args:
        base_dir: The base directory of the project.
        is_dev: Whether this is for development mode.

    Returns:
        The generated Python code as a string.
    """
    functions_dir = base_dir / "functions"
    agents_dir = base_dir / "agents"

    function_files = find_python_files(functions_dir)
    agent_files = find_python_files(agents_dir)

    # Generate imports
    function_imports: list[str] = []
    function_registrations: list[str] = []
    agent_imports: list[str] = []
    agent_registrations: list[str] = []

    # Generate function imports and registrations
    for idx, (name, path) in enumerate(function_files.items()):
        var_name = f"func_{idx}"
        if is_dev:
            # In dev mode, import from the actual path
            import_path = str(path.parent)
            module_name = path.stem
            function_imports.append(
                f"sys.path.insert(0, {repr(import_path)})\n"
                f"import {module_name} as {var_name}\n"
                f"sys.path.pop(0)"
            )
        else:
            # In production, import from built modules
            module_path = f"functions.{name.replace('/', '.')}"
            function_imports.append(f"from {module_path} import default as {var_name}")

        function_registrations.append(f'''
    # Register function: {name}
    fn = {var_name}
    if hasattr(fn, 'provider_controller') and fn.provider_controller:
        add_provider(fn.provider_controller)
    if hasattr(fn, 'function_metadata') and hasattr(fn, 'provider_controller') and hasattr(fn, 'handler'):
        provider_type_id = fn.provider_controller.type_id

        def make_invoke_callback(fn_handler, input_schema):
            def invoke_callback(req: InvokeFunctionRequest) -> InvokeFunctionResponse:
                try:
                    import json
                    params = json.loads(req.parameters)
                    # Parse input using pydantic model if available
                    if hasattr(input_schema, 'model_validate'):
                        parsed_input = input_schema.model_validate(params)
                    else:
                        parsed_input = params
                    import asyncio
                    result = asyncio.get_event_loop().run_until_complete(fn_handler(parsed_input))
                    # Serialize output using pydantic model if available
                    if hasattr(result, 'model_dump_json'):
                        return InvokeFunctionResponse.success(result.model_dump_json())
                    else:
                        return InvokeFunctionResponse.success(json.dumps(result))
                except Exception as e:
                    return InvokeFunctionResponse.failure(str(e))
            return invoke_callback

        update_function(provider_type_id, fn.function_metadata, make_invoke_callback(fn.handler, fn.input_schema))
''')

    # Generate agent imports and registrations
    for idx, (name, path) in enumerate(agent_files.items()):
        var_name = f"agent_{idx}"
        if is_dev:
            import_path = str(path.parent)
            module_name = path.stem
            agent_imports.append(
                f"sys.path.insert(0, {repr(import_path)})\n"
                f"import {module_name} as {var_name}\n"
                f"sys.path.pop(0)"
            )
        else:
            module_path = f"agents.{name.replace('/', '.')}"
            agent_imports.append(f"from {module_path} import default as {var_name}")

        agent_registrations.append(f'''
    # Register agent: {name}
    agent = {var_name}
    if hasattr(agent, 'agent_id') and hasattr(agent, 'project_id') and hasattr(agent, 'name') and hasattr(agent, 'description'):
        update_agent(Agent(
            agent.agent_id,
            agent.project_id,
            agent.name,
            agent.description,
        ))
''')

    has_agents = len(agent_files) > 0

    # Generate the restate service code if there are agents
    restate_code = ""
    if has_agents:
        restate_services = []
        for idx in range(len(agent_files)):
            var_name = f"agent_{idx}"
            restate_services.append(f'''        restate_sdk.object(
            name=f"{{agent_{idx}.project_id}}.{{agent_{idx}.agent_id}}",
            handlers={{
                "entrypoint": wrap_handler(agent_{idx}.entrypoint, agent_{idx}),
            }},
        )''')

        restate_code = f'''
    # Restate agent services
    from trysoma_sdk.agent import HandlerParams

    async def wrap_handler(handler, agent):
        async def wrapped(ctx, input_data):
            from trysoma_api_client import V1Api, Configuration
            soma = V1Api(Configuration(
                host=os.environ.get("SOMA_SERVER_BASE_URL", "http://localhost:3000")
            ))
            await handler(HandlerParams(
                ctx=ctx,
                soma=soma,
                task_id=input_data["taskId"],
                context_id=input_data["contextId"],
            ))
        return wrapped

    try:
        import restate_sdk

        restate_service_port = os.environ.get("RESTATE_SERVICE_PORT")
        if not restate_service_port:
            raise RuntimeError("RESTATE_SERVICE_PORT environment variable is not set")

        restate_port = int(restate_service_port)
        print(f"Starting Restate server on port {{restate_port}}...")

        endpoint = restate_sdk.Endpoint()
        for service in [
{chr(10).join(restate_services)}
        ]:
            endpoint.bind(service)

        # Start the Restate server
        await endpoint.serve(port=restate_port)
    except ImportError:
        print("Restate SDK not available, skipping agent server startup")
'''

    # Generate the full standalone server code
    return f'''#!/usr/bin/env python3
"""Auto-generated standalone server for Soma SDK."""

import asyncio
import json
import os
import sys
import signal

# Add project root to path for imports
sys.path.insert(0, {repr(str(base_dir))})

from trysoma_sdk import (
    add_provider,
    update_function,
    update_agent,
    start_grpc_server,
    set_secret_handler,
    set_environment_variable_handler,
    set_unset_secret_handler,
    set_unset_environment_variable_handler,
    resync_sdk,
    FunctionMetadata,
    InvokeFunctionRequest,
    InvokeFunctionResponse,
    Agent,
)

{chr(10).join(function_imports)}

{chr(10).join(agent_imports)}

print("SDK server starting...")


async def main():
    # Start gRPC server
    socket_path = os.environ.get("SOMA_SERVER_SOCK", "/tmp/soma-sdk.sock")
    project_dir = os.getcwd()

    grpc_task = asyncio.create_task(start_grpc_server(socket_path, project_dir))

    # Wait a bit for server to initialize
    await asyncio.sleep(0.1)
    print(f"gRPC server started on {{socket_path}}")

    # Register secret handler
    print("[INFO] Registering secret handler...")

    async def secret_handler(err, secrets):
        if err:
            print(f"Error in secret handler: {{err}}")
            return {{"error": {{"message": err.message}}}}

        secret_keys = [s.key for s in secrets]
        print(f"[INFO] Secret handler invoked with {{len(secrets)}} secrets: {{', '.join(secret_keys)}}")

        for secret in secrets:
            os.environ[secret.key] = secret.value
            print(f"[INFO] Set os.environ.{{secret.key}}")

        message = f"Injected {{len(secrets)}} secrets into os.environ"
        print(f"[INFO] Secret handler completed: {{message}}")
        return {{"data": {{"message": message}}}}

    set_secret_handler(secret_handler)
    print("[INFO] Secret handler registered successfully")

    # Register environment variable handler
    print("[INFO] Registering environment variable handler...")

    async def env_var_handler(err, env_vars):
        if err:
            print(f"Error in environment variable handler: {{err}}")
            return {{"error": {{"message": err.message}}}}

        env_var_keys = [e.key for e in env_vars]
        print(f"[INFO] Environment variable handler invoked with {{len(env_vars)}} env vars: {{', '.join(env_var_keys)}}")

        for env_var in env_vars:
            os.environ[env_var.key] = env_var.value
            print(f"[INFO] Set os.environ.{{env_var.key}}")

        message = f"Injected {{len(env_vars)}} environment variables into os.environ"
        print(f"[INFO] Environment variable handler completed: {{message}}")
        return {{"data": {{"message": message}}}}

    set_environment_variable_handler(env_var_handler)
    print("[INFO] Environment variable handler registered successfully")

    # Register unset secret handler
    print("[INFO] Registering unset secret handler...")

    async def unset_secret_handler(err, key):
        if err:
            print(f"Error in unset secret handler: {{err}}")
            return {{"error": {{"message": err.message}}}}

        print(f"[INFO] Unset secret handler invoked with key: {{key}}")
        if key in os.environ:
            del os.environ[key]
        print(f"[INFO] Removed os.environ.{{key}}")

        message = f"Removed secret '{{key}}' from os.environ"
        print(f"[INFO] Unset secret handler completed: {{message}}")
        return {{"data": {{"message": message}}}}

    set_unset_secret_handler(unset_secret_handler)
    print("[INFO] Unset secret handler registered successfully")

    # Register unset environment variable handler
    print("[INFO] Registering unset environment variable handler...")

    async def unset_env_var_handler(err, key):
        if err:
            print(f"Error in unset environment variable handler: {{err}}")
            return {{"error": {{"message": err.message}}}}

        print(f"[INFO] Unset environment variable handler invoked with key: {{key}}")
        if key in os.environ:
            del os.environ[key]
        print(f"[INFO] Removed os.environ.{{key}}")

        message = f"Removed environment variable '{{key}}' from os.environ"
        print(f"[INFO] Unset environment variable handler completed: {{message}}")
        return {{"data": {{"message": message}}}}

    set_unset_environment_variable_handler(unset_env_var_handler)
    print("[INFO] Unset environment variable handler registered successfully")

    # Register all providers and functions
{chr(10).join(function_registrations)}

    # Register all agents
{chr(10).join(agent_registrations)}

    print("SDK server ready!")

    # Trigger resync with API server
    max_retries = 10
    base_delay_ms = 500
    resync_success = False

    for attempt in range(1, max_retries + 1):
        if resync_success:
            break
        print(f"Triggering resync with API server (attempt {{attempt}}/{{max_retries}})...")
        try:
            await resync_sdk()
            print("Resync with API server completed successfully")
            resync_success = True
        except Exception as error:
            if attempt < max_retries:
                delay_ms = base_delay_ms * attempt
                print(f"Resync failed, retrying in {{delay_ms}}ms...")
                await asyncio.sleep(delay_ms / 1000)
            else:
                print(f"Failed to resync with API server after all retries: {{error}}")

{restate_code}

    # Keep the process alive
    stop_event = asyncio.Event()

    def handle_signal(signum, frame):
        print("\\nShutting down...")
        stop_event.set()

    signal.signal(signal.SIGINT, handle_signal)
    signal.signal(signal.SIGTERM, handle_signal)

    await stop_event.wait()


if __name__ == "__main__":
    asyncio.run(main())
'''


def generate_standalone(base_dir: str | Path, is_dev: bool = False) -> None:
    """Generate the standalone.py file.

    Args:
        base_dir: The base directory of the project.
        is_dev: Whether this is for development mode.
    """
    base_dir = Path(base_dir)
    soma_dir = base_dir / ".soma"
    soma_dir.mkdir(exist_ok=True)

    standalone_path = soma_dir / "standalone.py"
    content = generate_standalone_server(base_dir, is_dev)
    standalone_path.write_text(content)

    print(f"Generated {standalone_path}")


def watch_and_regenerate(base_dir: str | Path) -> None:
    """Watch for changes and regenerate standalone.py.

    Uses watchfiles to watch the functions/ and agents/ directories
    for changes and regenerates standalone.py when files change.

    Args:
        base_dir: The base directory of the project.
    """
    try:
        from watchfiles import watch, Change
    except ImportError:
        print("watchfiles not installed. Run: pip install watchfiles")
        sys.exit(1)

    base_dir = Path(base_dir)
    functions_dir = base_dir / "functions"
    agents_dir = base_dir / "agents"

    # Generate initial standalone.py
    generate_standalone(base_dir, is_dev=True)

    print(f"Watching {functions_dir} and {agents_dir} for changes...")

    # Watch both directories
    paths_to_watch = []
    if functions_dir.exists():
        paths_to_watch.append(str(functions_dir))
    if agents_dir.exists():
        paths_to_watch.append(str(agents_dir))

    if not paths_to_watch:
        print("No functions/ or agents/ directories found. Creating empty directories...")
        functions_dir.mkdir(exist_ok=True)
        agents_dir.mkdir(exist_ok=True)
        paths_to_watch = [str(functions_dir), str(agents_dir)]

    for changes in watch(*paths_to_watch):
        for change_type, path in changes:
            if path.endswith(".py") and not path.endswith("__pycache__"):
                change_name = {
                    Change.added: "Added",
                    Change.modified: "Modified",
                    Change.deleted: "Deleted",
                }.get(change_type, "Changed")
                print(f"{change_name}: {path}")
                generate_standalone(base_dir, is_dev=True)
                break  # Only regenerate once per batch of changes


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Soma SDK standalone server generator")
    parser.add_argument(
        "--watch",
        "-w",
        action="store_true",
        help="Watch for changes and regenerate",
    )
    parser.add_argument(
        "--dev",
        "-d",
        action="store_true",
        help="Generate for development mode",
    )
    parser.add_argument(
        "base_dir",
        nargs="?",
        default=".",
        help="Base directory of the project",
    )

    args = parser.parse_args()

    if args.watch:
        watch_and_regenerate(args.base_dir)
    else:
        generate_standalone(args.base_dir, is_dev=args.dev)
