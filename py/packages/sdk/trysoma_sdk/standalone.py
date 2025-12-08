"""Standalone server generation for Soma Python SDK.

This module is the Python equivalent of vite.ts in the JS SDK.
It generates a standalone.py file that runs the SDK server.
"""

import os
import sys
import types
from pathlib import Path


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
                f"from {module_name} import default as {var_name}  "
                f"# type: ignore  # pyright: ignore  # noqa: E402\n"
                f"sys.path.pop(0)"
            )
        else:
            # In production, import from built modules
            module_path = f"functions.{name.replace('/', '.')}"
            function_imports.append(f"from {module_path} import default as {var_name}")

        function_registrations.append(f"""
    # Register function: {name}
    fn = {var_name}
    if hasattr(fn, 'provider_controller') and fn.provider_controller:
        add_provider(fn.provider_controller)
    if (hasattr(fn, 'function_metadata') and hasattr(fn, 'provider_controller')
            and hasattr(fn, 'handler')):
        provider_type_id = getattr(fn.provider_controller, 'type_id')

        def make_invoke_callback(
            fn_handler: Callable[[object], Awaitable[object]],
            input_schema: type[BaseModel] | type[object]
        ) -> Callable[[InvokeFunctionRequest], InvokeFunctionResponse]:
            def invoke_callback(req: InvokeFunctionRequest) -> InvokeFunctionResponse:
                try:
                    import json
                    params = json.loads(getattr(req, 'parameters'))
                    # Parse input using pydantic model if available
                    if hasattr(input_schema, 'model_validate'):
                        schema = cast(type[BaseModel], input_schema)
                        parsed_input = schema.model_validate(params)
                    else:
                        parsed_input = params
                    loop = asyncio.get_event_loop()
                    result = loop.run_until_complete(fn_handler(parsed_input))
                    # Serialize output using pydantic model if available
                    if hasattr(result, 'model_dump_json'):
                        from pydantic import BaseModel as PydanticBaseModel
                        pydantic_result = cast(PydanticBaseModel, result)
                        return InvokeFunctionResponse.success(
                            pydantic_result.model_dump_json()
                        )
                    else:
                        return InvokeFunctionResponse.success(json.dumps(result))
                except Exception as e:
                    return InvokeFunctionResponse.failure(str(e))
            return invoke_callback

        update_function(
            provider_type_id,
            fn.function_metadata,
            make_invoke_callback(fn.handler, fn.input_schema)
        )
""")

    # Generate agent imports and registrations
    for idx, (name, path) in enumerate(agent_files.items()):
        var_name = f"agent_{idx}"
        if is_dev:
            import_path = str(path.parent)
            module_name = path.stem
            agent_imports.append(
                f"sys.path.insert(0, {repr(import_path)})\n"
                f"from {module_name} import default as {var_name}  "
                f"# type: ignore  # pyright: ignore  # noqa: E402\n"
                f"sys.path.pop(0)"
            )
        else:
            module_path = f"agents.{name.replace('/', '.')}"
            agent_imports.append(f"from {module_path} import default as {var_name}")

        agent_registrations.append(f"""
    # Register agent: {name}
    agent = {var_name}
    if (hasattr(agent, 'agent_id') and hasattr(agent, 'project_id')
            and hasattr(agent, 'name') and hasattr(agent, 'description')):
        add_agent(Agent(
            agent.agent_id,
            agent.project_id,
            agent.name,
            agent.description,
        ))
""")

    has_agents = len(agent_files) > 0

    # Generate the restate service code if there are agents
    restate_code = ""
    if has_agents:
        restate_services = []
        for idx in range(len(agent_files)):
            var_name = f"agent_{idx}"
            restate_services.append(
                f"""        # Create Virtual Object for agent
        agent_{idx}_object = restate.VirtualObject(
            f"{{agent_{idx}.project_id}}.{{agent_{idx}.agent_id}}"
        )
        
        @agent_{idx}_object.handler("entrypoint")
        async def agent_{idx}_entrypoint(
            ctx: restate.ObjectContext,
            input_data: dict[str, str]
        ) -> None:
            await wrap_handler(
                agent_{idx}.entrypoint, agent_{idx}
            )(ctx, input_data)"""
            )

        restate_code = f'''
    # Restate agent services
    from trysoma_sdk import HandlerParams

    def wrap_handler(
        handler: Callable[[HandlerParams], Awaitable[None]],
        agent: object
    ) -> Callable[["restate.ObjectContext", dict[str, str]], Awaitable[None]]:
        async def wrapped(
            ctx: "restate.ObjectContext",
            input_data: dict[str, str]
        ) -> None:
            from trysoma_api_client import V1Api
            from trysoma_api_client.configuration import Configuration
            from trysoma_api_client.api_client import ApiClient
            config = Configuration(
                host=os.environ.get("SOMA_SERVER_BASE_URL", "http://localhost:3000")
            )
            soma = V1Api(ApiClient(configuration=config))
            await handler(HandlerParams(
                ctx=ctx,
                soma=soma,
                task_id=input_data["taskId"],
                context_id=input_data["contextId"],
            ))
        return wrapped

    try:
        import restate
        import hypercorn.asyncio
        from hypercorn import Config
        import socket

        restate_service_port = os.environ.get("RESTATE_SERVICE_PORT")
        if not restate_service_port:
            raise RuntimeError("RESTATE_SERVICE_PORT environment variable is not set")

        restate_port = int(restate_service_port)
        print(f"Starting Restate server on port {{restate_port}}...")

        # Create Virtual Objects for agents
{chr(10).join(restate_services)}

        # Create Restate app with services
        restate_services_list = [
{chr(10).join([f"        agent_{idx}_object" for idx in range(len(agent_files))])}
        ]
        app = restate.app(services=restate_services_list)

        # Helper function to check if port is available (not in use)
        async def wait_for_port_free(port: int, max_wait_seconds: int = 30) -> None:
            """Wait for a port to become free (not in use)."""
            import time
            start_time = time.time()
            while time.time() - start_time < max_wait_seconds:
                sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                sock.settimeout(0.1)
                result = sock.connect_ex(('127.0.0.1', port))
                sock.close()
                if result != 0:  # Port is free
                    return
                await asyncio.sleep(0.1)
            raise RuntimeError(f"Port {{port}} did not become free within {{max_wait_seconds}} seconds")

        # Helper function to wait for port to be listening
        async def wait_for_port_listening(port: int, max_wait_seconds: int = 30) -> None:
            """Wait for a port to start accepting connections."""
            import time
            start_time = time.time()
            while time.time() - start_time < max_wait_seconds:
                sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                sock.settimeout(0.1)
                result = sock.connect_ex(('127.0.0.1', port))
                sock.close()
                if result == 0:  # Port is listening
                    return
                await asyncio.sleep(0.1)
            raise RuntimeError(f"Port {{port}} did not start listening within {{max_wait_seconds}} seconds")

        # Wait for port to be free (in case previous instance is shutting down)
        try:
            await wait_for_port_free(restate_port)
        except RuntimeError as e:
            print(f"Error: {{e}}")
            print(f"Attempting to kill process on port {{restate_port}}...")
            import subprocess
            try:
                # Find and kill the process using the port
                result = subprocess.run(
                    ["lsof", "-ti", f":{{restate_port}}"],
                    capture_output=True,
                    text=True
                )
                if result.stdout.strip():
                    pids = result.stdout.strip().split("\\n")
                    for pid in pids:
                        if pid:
                            subprocess.run(["kill", "-9", pid], check=False)
                            print(f"Killed process {{pid}}")
                    # Wait a bit for the port to be released
                    await asyncio.sleep(1.0)
            except Exception as kill_error:
                print(f"Failed to kill process: {{kill_error}}")
                print(f"Please manually kill the process using port {{restate_port}} and restart")
                raise SystemExit(1)

        # Create shutdown event for graceful Hypercorn shutdown
        # This event is used as the shutdown_trigger parameter for Hypercorn
        shutdown_event = asyncio.Event()

        def trigger_shutdown() -> None:
            \"\"\"Signal handler to trigger graceful shutdown.\"\"\"
            print("\\n[Shutdown] Received shutdown signal, initiating graceful shutdown...")
            shutdown_event.set()

        # Register signal handlers to trigger graceful shutdown
        loop = asyncio.get_event_loop()
        for sig in (signal.SIGINT, signal.SIGTERM):
            try:
                loop.add_signal_handler(sig, trigger_shutdown)
            except NotImplementedError:
                # Windows doesn't support add_signal_handler
                signal.signal(sig, lambda s, f: trigger_shutdown())

        # Start server on main thread
        conf = Config()
        conf.bind = [f"127.0.0.1:{{restate_port}}"]
        # Configure Hypercorn for HTTP/2 as required by Restate
        # See: https://docs.restate.dev/develop/python/serving#hypercorn
        conf.h2_max_concurrent_streams = 2147483647
        conf.keep_alive_max_requests = 2147483647
        conf.keep_alive_timeout = 2147483647
        # Set graceful timeout for shutdown
        conf.graceful_timeout = 5.0

        print(f"[Restate] Starting Restate server on port {{restate_port}}...")

        # Start server in background task with shutdown_trigger
        server_task = None

        async def start_server() -> None:
            print("[Restate] Hypercorn server task starting...")
            try:
                # Pass shutdown_trigger to Hypercorn for graceful shutdown
                await hypercorn.asyncio.serve(
                    app,
                    conf,
                    shutdown_trigger=shutdown_event.wait
                )
                print("[Restate] Hypercorn server stopped gracefully")
            except Exception as e:
                print(f"[Restate] Error in Hypercorn server: {{e}}")
                raise
            finally:
                # Clean up gRPC service on shutdown
                print("[Shutdown] Cleaning up gRPC service...")
                try:
                    kill_grpc_service()
                    print("[Shutdown] gRPC service killed successfully")
                except Exception as cleanup_error:
                    print(f"[Shutdown] Error cleaning up gRPC service: {{cleanup_error}}")

        server_task = asyncio.create_task(start_server())
        print(f"[Restate] Server task created, waiting for port {{restate_port}} to be listening...")

        # Wait for server to be listening
        await wait_for_port_listening(restate_port)
        print(f"[Restate] ✓ Server is listening on port {{restate_port}}")

        # Give the server a moment to fully initialize HTTP/2 endpoints
        print("[Restate] Waiting 1 second for HTTP/2 endpoints to initialize...")
        await asyncio.sleep(1.0)
        print("[Restate] HTTP/2 initialization complete")

        # Trigger resync in a separate thread to avoid blocking the asyncio event loop
        # Use threading.Thread instead of asyncio task to ensure it doesn't interfere
        import threading
        import time

        def trigger_resync_thread() -> None:
            """Trigger resync with API server in a separate thread."""
            try:
                # Additional delay to ensure server is fully ready before resync
                time.sleep(0.5)
                print("[Resync] Starting resync in background thread...")

                # Create a new event loop for this thread since resync_sdk is async
                async def run_resync() -> None:
                    max_retries = 10
                    base_delay_ms = 500
                    resync_success = False

                    for attempt in range(1, max_retries + 1):
                        if resync_success:
                            break
                        print(
                            f"[Resync] Triggering resync with API server "
                            f"(attempt {{attempt}}/{{max_retries}})..."
                        )
                        try:
                            # resync_sdk is now async, so we await it
                            print("[Resync] Calling resync_sdk()...")
                            await resync_sdk()
                            print("[Resync] ✓ Resync with API server completed successfully")
                            resync_success = True
                        except Exception as error:
                            print(f"[Resync] Resync attempt {{attempt}} failed: {{error}}")
                            if attempt < max_retries:
                                delay_ms = base_delay_ms * attempt
                                print(f"[Resync] Retrying in {{delay_ms}}ms...")
                                await asyncio.sleep(delay_ms / 1000)
                            else:
                                print(f"[Resync] ✗ Initial resync failed after {{max_retries}} attempts: {{error}}")
                                print("[Resync] This is OK - the server is running and resync will succeed on retry (e.g., on file changes)")
                                # Don't exit - server is running, resync will work later

                # Run the async function in a new event loop for this thread
                asyncio.run(run_resync())
            except Exception as e:
                # Catch any unexpected errors in the resync thread to prevent it from crashing
                print(f"[Resync] Unexpected error in resync thread: {{e}}")
                import traceback
                traceback.print_exc()

        # Start resync in a daemon thread - completely separate from asyncio event loop
        print("[Resync] Starting resync thread...")
        try:
            resync_thread = threading.Thread(target=trigger_resync_thread, daemon=True)
            resync_thread.start()
            print("[Resync] Resync thread started successfully")
        except Exception as e:
            print(f"[Resync] Failed to start resync thread: {{e}}")
            import traceback
            traceback.print_exc()
            # Don't fail server startup if resync thread can't be created

        # Keep server running until shutdown is triggered
        print("[Restate] Server is running, awaiting server task...")
        await server_task
        print("[Shutdown] Server shutdown complete")
    except ImportError:
        print("Restate SDK not available, skipping agent server startup")

        # Trigger resync with API server (no agents case)
        max_retries = 10
        base_delay_ms = 500
        resync_success = False

        for attempt in range(1, max_retries + 1):
            if resync_success:
                break
            print(
                f"Triggering resync with API server "
                f"(attempt {{attempt}}/{{max_retries}})..."
            )
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

        # Keep the process alive if Restate is not available
        stop_event = asyncio.Event()

        def handle_signal(
            signum: int, frame: types.FrameType | None
        ) -> None:
            print("\\nShutting down...")
            # Clean up gRPC service on shutdown
            try:
                kill_grpc_service()
                print("gRPC service killed successfully")
            except Exception as cleanup_error:
                print(f"Error cleaning up gRPC service: {{cleanup_error}}")
            stop_event.set()

        signal.signal(signal.SIGINT, handle_signal)
        signal.signal(signal.SIGTERM, handle_signal)

        await stop_event.wait()
'''

    # Generate the full standalone server code
    return f'''#!/usr/bin/env python3
"""Auto-generated standalone server for Soma SDK."""

import asyncio
import os
import sys
import signal
import types
from typing import Awaitable, Callable, cast

from pydantic import BaseModel

# Add project root and soma to path for imports
sys.path.insert(0, {repr(str(base_dir))})
_soma_dir = os.path.join({repr(str(base_dir))}, "soma")
if _soma_dir not in sys.path:
    sys.path.insert(0, _soma_dir)

from trysoma_sdk import (  # noqa: E402
    add_provider,
    update_function,
    add_agent,
    update_agent,
    start_grpc_server,
    kill_grpc_service,
    set_secret_handler,
    set_environment_variable_handler,
    set_unset_secret_handler,
    set_unset_environment_variable_handler,
    resync_sdk,
    InvokeFunctionRequest,
    InvokeFunctionResponse,
    Agent,
    Secret,
    EnvironmentVariable,
    SetSecretsResponse,
    SetEnvironmentVariablesResponse,
    UnsetSecretResponse,
    UnsetEnvironmentVariableResponse,
)

{chr(10).join(function_imports)}

{chr(10).join(agent_imports)}

print("SDK server starting...")


async def main() -> None:
    # Start gRPC server
    socket_path = os.environ.get("SOMA_SERVER_SOCK", "/tmp/soma-sdk.sock")
    project_dir = os.getcwd()

    await start_grpc_server(socket_path, project_dir)

    print(f"gRPC server started on {{socket_path}}")

    # Register secret handler
    print("[INFO] Registering secret handler...")

    def secret_handler(secrets: list[Secret]) -> SetSecretsResponse:
        secret_keys = [getattr(s, 'key') for s in secrets]
        print(
            f"[INFO] Secret handler invoked with {{len(secrets)}} secrets: "
            f"{{', '.join(secret_keys)}}"
        )

        for secret in secrets:
            os.environ[getattr(secret, 'key')] = getattr(secret, 'value')
            print(f"[INFO] Set os.environ.{{getattr(secret, 'key')}}")

        message = f"Injected {{len(secrets)}} secrets into os.environ"
        print(f"[INFO] Secret handler completed: {{message}}")
        from trysoma_sdk_core import SetSecretsSuccess
        return SetSecretsResponse(data=SetSecretsSuccess(message))

    set_secret_handler(secret_handler)
    print("[INFO] Secret handler registered successfully")

    # Register environment variable handler
    print("[INFO] Registering environment variable handler...")

    def env_var_handler(
        env_vars: list[EnvironmentVariable]
    ) -> SetEnvironmentVariablesResponse:
        env_var_keys = [getattr(e, 'key') for e in env_vars]
        print(
            f"[INFO] Environment variable handler invoked with "
            f"{{len(env_vars)}} env vars: {{', '.join(env_var_keys)}}"
        )

        for env_var in env_vars:
            os.environ[getattr(env_var, 'key')] = getattr(env_var, 'value')
            print(f"[INFO] Set os.environ.{{getattr(env_var, 'key')}}")

        message = (
            f"Injected {{len(env_vars)}} environment variables "
            f"into os.environ"
        )
        print(f"[INFO] Environment variable handler completed: {{message}}")
        from trysoma_sdk_core import SetEnvironmentVariablesSuccess
        return SetEnvironmentVariablesResponse(
            data=SetEnvironmentVariablesSuccess(message)
        )

    set_environment_variable_handler(env_var_handler)
    print("[INFO] Environment variable handler registered successfully")

    # Register unset secret handler
    print("[INFO] Registering unset secret handler...")

    def unset_secret_handler(key: str) -> UnsetSecretResponse:
        print(f"[INFO] Unset secret handler invoked with key: {{key}}")
        if key in os.environ:
            del os.environ[key]
        print(f"[INFO] Removed os.environ.{{key}}")

        message = f"Removed secret '{{key}}' from os.environ"
        print(f"[INFO] Unset secret handler completed: {{message}}")
        from trysoma_sdk_core import UnsetSecretSuccess
        return UnsetSecretResponse(data=UnsetSecretSuccess(message))

    set_unset_secret_handler(unset_secret_handler)
    print("[INFO] Unset secret handler registered successfully")

    # Register unset environment variable handler
    print("[INFO] Registering unset environment variable handler...")

    def unset_env_var_handler(key: str) -> UnsetEnvironmentVariableResponse:
        print(f"[INFO] Unset environment variable handler invoked with key: {{key}}")
        if key in os.environ:
            del os.environ[key]
        print(f"[INFO] Removed os.environ.{{key}}")

        message = (
            f"Removed environment variable '{{key}}' from os.environ"
        )
        print(
            f"[INFO] Unset environment variable handler completed: "
            f"{{message}}"
        )
        from trysoma_sdk_core import UnsetEnvironmentVariableSuccess
        return UnsetEnvironmentVariableResponse(
            data=UnsetEnvironmentVariableSuccess(message)
        )

    set_unset_environment_variable_handler(unset_env_var_handler)
    print("[INFO] Unset environment variable handler registered successfully")

    # Register all providers and functions
{chr(10).join(function_registrations)}

    # Register all agents
{chr(10).join(agent_registrations)}

    print("SDK server ready!")

{restate_code}
{
        (
            chr(10)
            + chr(10)
            + "    # Keep the process alive (only if no agents/Restate server)"
            + chr(10)
            + "    stop_event = asyncio.Event()"
            + chr(10)
            + chr(10)
            + "    def handle_signal("
            + chr(10)
            + "        signum: int, frame: types.FrameType | None"
            + chr(10)
            + "    ) -> None:"
            + chr(10)
            + '        print("'
            + chr(10)
            + 'Shutting down...")'
            + chr(10)
            + "        stop_event.set()"
            + chr(10)
            + chr(10)
            + "    signal.signal(signal.SIGINT, handle_signal)"
            + chr(10)
            + "    signal.signal(signal.SIGTERM, handle_signal)"
            + chr(10)
            + chr(10)
            + "    await stop_event.wait()"
            if not has_agents
            else ""
        )
    }


if __name__ == "__main__":
    # Suppress RuntimeWarning about module being found in sys.modules
    # This can happen when running standalone.py that imports from trysoma_sdk package
    import warnings
    warnings.filterwarnings("ignore", category=RuntimeWarning, message=".*found in sys.modules.*")
    
    print("Starting standalone server...")
    asyncio.run(main())
'''


def generate_standalone(base_dir: str | Path, is_dev: bool = False) -> None:
    """Generate the standalone.py file.

    Args:
        base_dir: The base directory of the project.
        is_dev: Whether this is for development mode.
    """
    base_dir = Path(base_dir)
    soma_dir = base_dir / "soma"
    soma_dir.mkdir(exist_ok=True)

    standalone_path = soma_dir / "standalone.py"
    content = generate_standalone_server(base_dir, is_dev)
    standalone_path.write_text(content)

    # Create __init__.py in soma package
    init_path = soma_dir / "__init__.py"
    if not init_path.exists():
        init_path.write_text('"""Soma generated package."""\n')

    # Ensure bridge.py exists (even if empty) so imports don't fail
    bridge_path = soma_dir / "bridge.py"
    if not bridge_path.exists():
        bridge_path.write_text('''# Auto-generated Python bridge client
# DO NOT EDIT - This file is generated by the Soma SDK

from __future__ import annotations
import os
import httpx
from typing import Any, TYPE_CHECKING, TypeVar

if TYPE_CHECKING:
    from restate import ObjectContext

SOMA_SERVER_BASE_URL = os.environ.get("SOMA_SERVER_BASE_URL", "http://localhost:3000")

# Type variables for generic bridge function invocation
TParams = TypeVar("TParams", bound=dict[str, Any])
TResult = TypeVar("TResult")


async def _invoke_bridge_function(
    ctx: "ObjectContext",
    provider_instance_id: str,
    function_controller_type_id: str,
    params: TParams,
) -> TResult:
    """Internal helper to invoke a bridge function via the Soma API."""
    url = f"{SOMA_SERVER_BASE_URL}/api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/invoke"

    async def _make_request() -> TResult:
        async with httpx.AsyncClient() as client:
            response = await client.post(
                url,
                json=params,
                headers={"Content-Type": "application/json"},
            )
            response.raise_for_status()
            return response.json()

    # Use restate's run_typed() to make the call durable with proper typing
    return await ctx.run_typed("invoke_bridge_function", _make_request)


class Bridge:
    """Main bridge class providing access to all providers."""

    def __init__(self, ctx: "ObjectContext"):
        self._ctx = ctx


def get_bridge(ctx: "ObjectContext") -> Bridge:
    """Get a Bridge instance for the given Restate context."""
    return Bridge(ctx)
''')

    print(f"Generated {standalone_path}")


def clear_pycache(base_dir: Path) -> None:
    """Clear all __pycache__ directories and .pyc files.

    Args:
        base_dir: The base directory to clean.
    """
    import shutil

    # Find and remove all __pycache__ directories
    for pycache_dir in base_dir.rglob("__pycache__"):
        try:
            shutil.rmtree(pycache_dir)
        except OSError:
            pass

    # Find and remove all .pyc and .pyo files
    for pyc_file in base_dir.rglob("*.pyc"):
        try:
            pyc_file.unlink()
        except OSError:
            pass

    for pyo_file in base_dir.rglob("*.pyo"):
        try:
            pyo_file.unlink()
        except OSError:
            pass


def watch_and_regenerate(base_dir: str | Path) -> None:
    """Watch for changes and regenerate standalone.py, then run the server.

    Uses watchfiles to watch the functions/ and agents/ directories
    for changes and regenerates standalone.py when files change.
    Also starts the standalone server.

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
    standalone_path = base_dir / "soma" / "standalone.py"

    # Clear Python cache before generating
    print("Clearing Python cache...")
    clear_pycache(base_dir)

    # Also clear cache in the SDK package directory to ensure fresh imports
    sdk_package_dir = Path(__file__).parent
    clear_pycache(sdk_package_dir)

    # Generate initial standalone.py
    generate_standalone(base_dir, is_dev=True)

    # Start the server in a subprocess
    import subprocess
    import signal

    server_process: subprocess.Popen[bytes] | None = None
    socket_path = os.environ.get("SOMA_SERVER_SOCK", "/tmp/soma-sdk.sock")

    def wait_for_socket_released(timeout: float = 10.0) -> bool:
        """Wait for the Unix socket file to be released/removed."""
        import time

        start_time = time.time()
        while time.time() - start_time < timeout:
            if not Path(socket_path).exists():
                return True
            time.sleep(0.1)
        return False

    def wait_for_port_free(port: int, timeout: float = 10.0) -> bool:
        """Wait for a TCP port to be free."""
        import time
        import socket as sock_module

        start_time = time.time()
        while time.time() - start_time < timeout:
            s = sock_module.socket(sock_module.AF_INET, sock_module.SOCK_STREAM)
            s.settimeout(0.1)
            result = s.connect_ex(("127.0.0.1", port))
            s.close()
            if result != 0:  # Port is free (connection refused)
                return True
            time.sleep(0.1)
        return False

    def start_server() -> subprocess.Popen[bytes]:
        """Start the standalone server in its own process group."""
        # Start in a new process group so we can kill all children together
        return subprocess.Popen(
            [sys.executable, str(standalone_path)],
            cwd=str(base_dir),
            env=os.environ.copy(),
            start_new_session=True,  # Creates new process group
        )

    def kill_server_process_group(
        proc: subprocess.Popen[bytes], timeout: float = 5.0
    ) -> None:
        """Kill a server process and all its children using process group."""
        import time

        if proc.poll() is not None:
            # Process already dead
            return

        pgid = None
        try:
            pgid = os.getpgid(proc.pid)
        except (ProcessLookupError, OSError):
            # Process already gone
            return

        # First try SIGTERM to the entire process group
        try:
            os.killpg(pgid, signal.SIGTERM)
        except (ProcessLookupError, OSError):
            pass

        # Wait for graceful shutdown
        start_time = time.time()
        while time.time() - start_time < timeout:
            if proc.poll() is not None:
                return
            time.sleep(0.1)

        # If still alive, force kill the entire process group
        print("Process did not terminate gracefully, sending SIGKILL...")
        try:
            os.killpg(pgid, signal.SIGKILL)
        except (ProcessLookupError, OSError):
            pass

        # Final wait
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            pass

    def restart_server() -> None:
        """Restart the standalone server."""
        nonlocal server_process

        # Get the Restate port from environment
        restate_port_str = os.environ.get("RESTATE_SERVICE_PORT")
        restate_port = int(restate_port_str) if restate_port_str else None

        if server_process:
            print("Restarting SDK server...")
            kill_server_process_group(server_process)

            # Wait for the socket to be released before starting new server
            print("Waiting for socket to be released...")
            if wait_for_socket_released(timeout=5.0):
                print("Socket released.")
            else:
                # Force remove the socket file if it still exists
                socket_file = Path(socket_path)
                if socket_file.exists():
                    print("Force removing stale socket file...")
                    try:
                        socket_file.unlink()
                    except OSError as e:
                        print(f"Warning: Could not remove socket file: {e}")

            # Also wait for the Restate port to be free
            if restate_port:
                print(f"Waiting for port {restate_port} to be free...")
                if wait_for_port_free(restate_port, timeout=5.0):
                    print(f"Port {restate_port} is free.")
                else:
                    print(
                        f"Warning: Port {restate_port} still in use, new server may fail to bind."
                    )

            # Small delay to ensure everything is cleaned up
            import time

            time.sleep(0.3)

        server_process = start_server()

    # Start the server initially
    print("Starting SDK server...")
    server_process = start_server()

    def handle_signal(signum: int, frame: types.FrameType | None) -> None:
        """Handle shutdown signal."""
        print("\nShutting down...")
        if server_process:
            kill_server_process_group(server_process)
        sys.exit(0)

    signal.signal(signal.SIGINT, handle_signal)
    signal.signal(signal.SIGTERM, handle_signal)

    print(f"Watching {functions_dir} and {agents_dir} for changes...")

    # Watch both directories
    paths_to_watch = []
    if functions_dir.exists():
        paths_to_watch.append(str(functions_dir))
    if agents_dir.exists():
        paths_to_watch.append(str(agents_dir))

    if not paths_to_watch:
        print(
            "No functions/ or agents/ directories found. Creating empty directories..."
        )
        functions_dir.mkdir(exist_ok=True)
        agents_dir.mkdir(exist_ok=True)
        paths_to_watch = [str(functions_dir), str(agents_dir)]

    try:
        for changes in watch(*paths_to_watch):
            for change_type, path in changes:
                if path.endswith(".py") and not path.endswith("__pycache__"):
                    change_name = {
                        Change.added: "Added",
                        Change.modified: "Modified",
                        Change.deleted: "Deleted",
                    }.get(change_type, "Changed")
                    print(f"{change_name}: {path}")
                    # Clear cache before regenerating
                    clear_pycache(base_dir)
                    clear_pycache(Path(__file__).parent)
                    generate_standalone(base_dir, is_dev=True)
                    restart_server()
                    break  # Only regenerate once per batch of changes
    except KeyboardInterrupt:
        handle_signal(signal.SIGINT, None)


def dev_entrypoint() -> None:
    """Console script entrypoint for 'dev' command.

    This is the function called by the console script defined in pyproject.toml.
    It defaults to the current working directory.
    """
    watch_and_regenerate(".")


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
