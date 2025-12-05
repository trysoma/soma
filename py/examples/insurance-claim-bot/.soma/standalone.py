#!/usr/bin/env python3
"""Auto-generated standalone server for Soma SDK."""

import asyncio
import json
import os
import sys
import signal

# Add project root to path for imports
sys.path.insert(0, '.')

from soma_sdk import (
    add_function,
    add_provider,
    add_agent,
    start_grpc_server,
    set_secret_handler,
    set_environment_variable_handler,
    set_unset_secret_handler,
    set_unset_environment_variable_handler,
    resync_sdk,
)

from functions.approve_claim import default as func_0

from agents.index import default as agent_0

print("SDK server starting...")


async def main():
    # Start gRPC server
    socket_path = os.environ.get("SOMA_SERVER_SOCK", "/tmp/soma-sdk.sock")
    project_dir = os.getcwd()

    grpc_task = asyncio.create_task(start_grpc_server(socket_path, project_dir))

    # Wait a bit for server to initialize
    await asyncio.sleep(0.1)
    print(f"gRPC server started on {socket_path}")

    # Register secret handler
    print("[INFO] Registering secret handler...")

    async def secret_handler(err, secrets):
        if err:
            print(f"Error in secret handler: {err}")
            return {"error": {"message": err.message}}

        secret_keys = [s.key for s in secrets]
        print(f"[INFO] Secret handler invoked with {len(secrets)} secrets: {', '.join(secret_keys)}")

        for secret in secrets:
            os.environ[secret.key] = secret.value
            print(f"[INFO] Set os.environ.{secret.key}")

        message = f"Injected {len(secrets)} secrets into os.environ"
        print(f"[INFO] Secret handler completed: {message}")
        return {"data": {"message": message}}

    set_secret_handler(secret_handler)
    print("[INFO] Secret handler registered successfully")

    # Register environment variable handler
    print("[INFO] Registering environment variable handler...")

    async def env_var_handler(err, env_vars):
        if err:
            print(f"Error in environment variable handler: {err}")
            return {"error": {"message": err.message}}

        env_var_keys = [e.key for e in env_vars]
        print(f"[INFO] Environment variable handler invoked with {len(env_vars)} env vars: {', '.join(env_var_keys)}")

        for env_var in env_vars:
            os.environ[env_var.key] = env_var.value
            print(f"[INFO] Set os.environ.{env_var.key}")

        message = f"Injected {len(env_vars)} environment variables into os.environ"
        print(f"[INFO] Environment variable handler completed: {message}")
        return {"data": {"message": message}}

    set_environment_variable_handler(env_var_handler)
    print("[INFO] Environment variable handler registered successfully")

    # Register unset secret handler
    print("[INFO] Registering unset secret handler...")

    async def unset_secret_handler(err, key):
        if err:
            print(f"Error in unset secret handler: {err}")
            return {"error": {"message": err.message}}

        print(f"[INFO] Unset secret handler invoked with key: {key}")
        if key in os.environ:
            del os.environ[key]
        print(f"[INFO] Removed os.environ.{key}")

        message = f"Removed secret '{key}' from os.environ"
        print(f"[INFO] Unset secret handler completed: {message}")
        return {"data": {"message": message}}

    set_unset_secret_handler(unset_secret_handler)
    print("[INFO] Unset secret handler registered successfully")

    # Register unset environment variable handler
    print("[INFO] Registering unset environment variable handler...")

    async def unset_env_var_handler(err, key):
        if err:
            print(f"Error in unset environment variable handler: {err}")
            return {"error": {"message": err.message}}

        print(f"[INFO] Unset environment variable handler invoked with key: {key}")
        if key in os.environ:
            del os.environ[key]
        print(f"[INFO] Removed os.environ.{key}")

        message = f"Removed environment variable '{key}' from os.environ"
        print(f"[INFO] Unset environment variable handler completed: {message}")
        return {"data": {"message": message}}

    set_unset_environment_variable_handler(unset_env_var_handler)
    print("[INFO] Unset environment variable handler registered successfully")

    # Register all providers and functions

    # Register function: approve_claim
    fn = func_0
    if hasattr(fn, 'provider_controller') and fn.provider_controller:
        add_provider(fn.provider_controller)
    if hasattr(fn, 'function_controller') and hasattr(fn, 'provider_controller') and hasattr(fn, 'handler'):
        provider_type_id = fn.provider_controller.type_id
        function_metadata = {
            "name": fn.function_controller.name,
            "description": fn.function_controller.description,
            "parameters": fn.function_controller.parameters,
            "output": fn.function_controller.output,
        }

        async def make_invoke_callback(fn_handler):
            async def invoke_callback(err, req):
                if err:
                    return {"error": err.message}
                try:
                    import json
                    params = json.loads(req.parameters)
                    result = await fn_handler(params)
                    return {"data": json.dumps(result)}
                except Exception as e:
                    return {"error": str(e)}
            return invoke_callback

        add_function(provider_type_id, function_metadata, await make_invoke_callback(fn.handler))


    # Register all agents

    # Register agent: index
    agent = agent_0
    if hasattr(agent, 'agent_id') and hasattr(agent, 'project_id') and hasattr(agent, 'name') and hasattr(agent, 'description'):
        add_agent({
            "id": agent.agent_id,
            "project_id": agent.project_id,
            "name": agent.name,
            "description": agent.description,
        })


    print("SDK server ready!")

    # Trigger resync with API server
    max_retries = 10
    base_delay_ms = 500
    resync_success = False

    for attempt in range(1, max_retries + 1):
        if resync_success:
            break
        print(f"Triggering resync with API server (attempt {attempt}/{max_retries})...")
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


    # Restate agent services
    from soma_sdk.agent import HandlerParams

    async def wrap_handler(handler, agent):
        async def wrapped(ctx, input_data):
            from soma_api_client import V1Api, Configuration
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
        import restate

        restate_service_port = os.environ.get("RESTATE_SERVICE_PORT")
        if not restate_service_port:
            raise RuntimeError("RESTATE_SERVICE_PORT environment variable is not set")

        restate_port = int(restate_service_port)
        print(f"Starting Restate server on port {restate_port}...")

        endpoint = restate.Endpoint()
        for service in [
        restate.object(
            name=f"{agent_0.project_id}.{agent_0.agent_id}",
            handlers={
                "entrypoint": wrap_handler(agent_0.entrypoint, agent_0),
            },
        )
        ]:
            endpoint.bind(service)

        # Start the Restate server
        await endpoint.serve(port=restate_port)
    except ImportError:
        print("Restate SDK not available, skipping agent server startup")


    # Keep the process alive
    stop_event = asyncio.Event()

    def handle_signal(signum, frame):
        print("\nShutting down...")
        stop_event.set()

    signal.signal(signal.SIGINT, handle_signal)
    signal.signal(signal.SIGTERM, handle_signal)

    await stop_event.wait()


if __name__ == "__main__":
    asyncio.run(main())
