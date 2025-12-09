#!/usr/bin/env python3
"""Auto-generated standalone server for Soma SDK."""

import asyncio
import os
import sys
import signal
import types
from typing import Awaitable, Callable, cast

from pydantic import BaseModel

# Add project root and soma to path for imports
sys.path.insert(0, '.')
_soma_dir = os.path.join('.', "soma")
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

sys.path.insert(0, 'functions')
from approve_claim import default as func_0  # type: ignore  # pyright: ignore  # noqa: E402
sys.path.pop(0)

sys.path.insert(0, 'agents')
from index import default as agent_0  # type: ignore  # pyright: ignore  # noqa: E402
sys.path.pop(0)
sys.path.insert(0, 'agents')
from new import default as agent_1  # type: ignore  # pyright: ignore  # noqa: E402
sys.path.pop(0)

print("SDK server starting...")


async def main() -> None:
    # Start gRPC server
    socket_path = os.environ.get("SOMA_SERVER_SOCK", "/tmp/soma-sdk.sock")
    project_dir = os.getcwd()

    await start_grpc_server(socket_path, project_dir)

    print(f"gRPC server started on {socket_path}")

    # Register secret handler
    print("[INFO] Registering secret handler...")

    def secret_handler(secrets: list[Secret]) -> SetSecretsResponse:
        secret_keys = [getattr(s, 'key') for s in secrets]
        print(
            f"[INFO] Secret handler invoked with {len(secrets)} secrets: "
            f"{', '.join(secret_keys)}"
        )

        for secret in secrets:
            os.environ[getattr(secret, 'key')] = getattr(secret, 'value')
            print(f"[INFO] Set os.environ.{getattr(secret, 'key')}")

        message = f"Injected {len(secrets)} secrets into os.environ"
        print(f"[INFO] Secret handler completed: {message}")
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
            f"{len(env_vars)} env vars: {', '.join(env_var_keys)}"
        )

        for env_var in env_vars:
            os.environ[getattr(env_var, 'key')] = getattr(env_var, 'value')
            print(f"[INFO] Set os.environ.{getattr(env_var, 'key')}")

        message = (
            f"Injected {len(env_vars)} environment variables "
            f"into os.environ"
        )
        print(f"[INFO] Environment variable handler completed: {message}")
        from trysoma_sdk_core import SetEnvironmentVariablesSuccess
        return SetEnvironmentVariablesResponse(
            data=SetEnvironmentVariablesSuccess(message)
        )

    set_environment_variable_handler(env_var_handler)
    print("[INFO] Environment variable handler registered successfully")

    # Register unset secret handler
    print("[INFO] Registering unset secret handler...")

    def unset_secret_handler(key: str) -> UnsetSecretResponse:
        print(f"[INFO] Unset secret handler invoked with key: {key}")
        if key in os.environ:
            del os.environ[key]
        print(f"[INFO] Removed os.environ.{key}")

        message = f"Removed secret '{key}' from os.environ"
        print(f"[INFO] Unset secret handler completed: {message}")
        from trysoma_sdk_core import UnsetSecretSuccess
        return UnsetSecretResponse(data=UnsetSecretSuccess(message))

    set_unset_secret_handler(unset_secret_handler)
    print("[INFO] Unset secret handler registered successfully")

    # Register unset environment variable handler
    print("[INFO] Registering unset environment variable handler...")

    def unset_env_var_handler(key: str) -> UnsetEnvironmentVariableResponse:
        print(f"[INFO] Unset environment variable handler invoked with key: {key}")
        if key in os.environ:
            del os.environ[key]
        print(f"[INFO] Removed os.environ.{key}")

        message = (
            f"Removed environment variable '{key}' from os.environ"
        )
        print(
            f"[INFO] Unset environment variable handler completed: "
            f"{message}"
        )
        from trysoma_sdk_core import UnsetEnvironmentVariableSuccess
        return UnsetEnvironmentVariableResponse(
            data=UnsetEnvironmentVariableSuccess(message)
        )

    set_unset_environment_variable_handler(unset_env_var_handler)
    print("[INFO] Unset environment variable handler registered successfully")

    # Register all providers and functions

    # Register function: approve_claim
    fn = func_0
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


    # Register all agents

    # Register agent: index
    agent = agent_0
    if (hasattr(agent, 'agent_id') and hasattr(agent, 'project_id')
            and hasattr(agent, 'name') and hasattr(agent, 'description')):
        add_agent(Agent(
            agent.agent_id,
            agent.project_id,
            agent.name,
            agent.description,
        ))


    # Register agent: new
    agent = agent_1
    if (hasattr(agent, 'agent_id') and hasattr(agent, 'project_id')
            and hasattr(agent, 'name') and hasattr(agent, 'description')):
        add_agent(Agent(
            agent.agent_id,
            agent.project_id,
            agent.name,
            agent.description,
        ))


    print("SDK server ready!")


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
        print(f"Starting Restate server on port {restate_port}...")

        # Create Virtual Objects for agents
        # Create Virtual Object for agent
        agent_0_object = restate.VirtualObject(
            f"{agent_0.project_id}.{agent_0.agent_id}"
        )
        
        @agent_0_object.handler("entrypoint")
        async def agent_0_entrypoint(
            ctx: restate.ObjectContext,
            input_data: dict[str, str]
        ) -> None:
            await wrap_handler(
                agent_0.entrypoint, agent_0
            )(ctx, input_data)
        # Create Virtual Object for agent
        agent_1_object = restate.VirtualObject(
            f"{agent_1.project_id}.{agent_1.agent_id}"
        )
        
        @agent_1_object.handler("entrypoint")
        async def agent_1_entrypoint(
            ctx: restate.ObjectContext,
            input_data: dict[str, str]
        ) -> None:
            await wrap_handler(
                agent_1.entrypoint, agent_1
            )(ctx, input_data)

        # Create Restate app with services
        restate_services_list = [
        agent_0_object,
        agent_1_object,
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
            raise RuntimeError(f"Port {port} did not become free within {max_wait_seconds} seconds")

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
            raise RuntimeError(f"Port {port} did not start listening within {max_wait_seconds} seconds")

        # Wait for port to be free (in case previous instance is shutting down)
        try:
            await wait_for_port_free(restate_port)
        except RuntimeError as e:
            print(f"Error: {e}")
            print(f"Attempting to kill process on port {restate_port}...")
            import subprocess
            try:
                # Find and kill the process using the port
                result = subprocess.run(
                    ["lsof", "-ti", f":{restate_port}"],
                    capture_output=True,
                    text=True
                )
                if result.stdout.strip():
                    pids = result.stdout.strip().split("\n")
                    for pid in pids:
                        if pid:
                            subprocess.run(["kill", "-9", pid], check=False)
                            print(f"Killed process {pid}")
                    # Wait a bit for the port to be released
                    await asyncio.sleep(1.0)
            except Exception as kill_error:
                print(f"Failed to kill process: {kill_error}")
                print(f"Please manually kill the process using port {restate_port} and restart")
                raise SystemExit(1)

        # Create shutdown event for graceful Hypercorn shutdown
        # This event is used as the shutdown_trigger parameter for Hypercorn
        shutdown_event = asyncio.Event()

        def trigger_shutdown() -> None:
            """Signal handler to trigger graceful shutdown."""
            print("\n[Shutdown] Received shutdown signal, initiating graceful shutdown...")
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
        conf.bind = [f"127.0.0.1:{restate_port}"]
        # Configure Hypercorn for HTTP/2 as required by Restate
        # See: https://docs.restate.dev/develop/python/serving#hypercorn
        conf.h2_max_concurrent_streams = 2147483647
        conf.keep_alive_max_requests = 2147483647
        conf.keep_alive_timeout = 2147483647
        # Set graceful timeout for shutdown
        conf.graceful_timeout = 5.0

        print(f"[Restate] Starting Restate server on port {restate_port}...")

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
                print(f"[Restate] Error in Hypercorn server: {e}")
                raise
            finally:
                # Clean up gRPC service on shutdown
                print("[Shutdown] Cleaning up gRPC service...")
                try:
                    kill_grpc_service()
                    print("[Shutdown] gRPC service killed successfully")
                except Exception as cleanup_error:
                    print(f"[Shutdown] Error cleaning up gRPC service: {cleanup_error}")

        server_task = asyncio.create_task(start_server())
        print(f"[Restate] Server task created, waiting for port {restate_port} to be listening...")

        # Wait for server to be listening
        await wait_for_port_listening(restate_port)
        print(f"[Restate] ✓ Server is listening on port {restate_port}")

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
                            f"(attempt {attempt}/{max_retries})..."
                        )
                        try:
                            # resync_sdk is now async, so we await it
                            print("[Resync] Calling resync_sdk()...")
                            await resync_sdk()
                            print("[Resync] ✓ Resync with API server completed successfully")
                            resync_success = True
                        except Exception as error:
                            print(f"[Resync] Resync attempt {attempt} failed: {error}")
                            if attempt < max_retries:
                                delay_ms = base_delay_ms * attempt
                                print(f"[Resync] Retrying in {delay_ms}ms...")
                                await asyncio.sleep(delay_ms / 1000)
                            else:
                                print(f"[Resync] ✗ Initial resync failed after {max_retries} attempts: {error}")
                                print("[Resync] This is OK - the server is running and resync will succeed on retry (e.g., on file changes)")
                                # Don't exit - server is running, resync will work later

                # Run the async function in a new event loop for this thread
                asyncio.run(run_resync())
            except Exception as e:
                # Catch any unexpected errors in the resync thread to prevent it from crashing
                print(f"[Resync] Unexpected error in resync thread: {e}")
                import traceback
                traceback.print_exc()

        # Start resync in a daemon thread - completely separate from asyncio event loop
        print("[Resync] Starting resync thread...")
        try:
            resync_thread = threading.Thread(target=trigger_resync_thread, daemon=True)
            resync_thread.start()
            print("[Resync] Resync thread started successfully")
        except Exception as e:
            print(f"[Resync] Failed to start resync thread: {e}")
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
                f"(attempt {attempt}/{max_retries})..."
            )
            try:
                await resync_sdk()

                print("Resync with API server completed successfully")
                resync_success = True
            except Exception as error:
                if attempt < max_retries:
                    delay_ms = base_delay_ms * attempt
                    print(f"Resync failed, retrying in {delay_ms}ms...")
                    await asyncio.sleep(delay_ms / 1000)
                else:
                    print(f"Failed to resync with API server after all retries: {error}")

        # Keep the process alive if Restate is not available
        stop_event = asyncio.Event()

        def handle_signal(
            signum: int, frame: types.FrameType | None
        ) -> None:
            print("\nShutting down...")
            # Clean up gRPC service on shutdown
            try:
                kill_grpc_service()
                print("gRPC service killed successfully")
            except Exception as cleanup_error:
                print(f"Error cleaning up gRPC service: {cleanup_error}")
            stop_event.set()

        signal.signal(signal.SIGINT, handle_signal)
        signal.signal(signal.SIGTERM, handle_signal)

        await stop_event.wait()




if __name__ == "__main__":
    # Suppress RuntimeWarning about module being found in sys.modules
    # This can happen when running standalone.py that imports from trysoma_sdk package
    import warnings
    warnings.filterwarnings("ignore", category=RuntimeWarning, message=".*found in sys.modules.*")
    
    print("Starting standalone server...")
    asyncio.run(main())
