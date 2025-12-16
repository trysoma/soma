/// <reference types="node" />
// Auto-generated standalone server
import { addFunction, addProvider, addAgent, startGrpcServer, setSecretHandler, setEnvironmentVariableHandler, setUnsetSecretHandler, setUnsetEnvironmentVariableHandler, resyncSdk } from '@trysoma/sdk';
import type { Secret, SetSecretsResponse, SetSecretsSuccess, CallbackError, EnvironmentVariable, SetEnvironmentVariablesResponse, SetEnvironmentVariablesSuccess, UnsetSecretResponse, UnsetSecretSuccess, UnsetEnvironmentVariableResponse, UnsetEnvironmentVariableSuccess } from '@trysoma/sdk';
import * as restate from '@restatedev/restate-sdk';
import * as http2 from 'http2';

import func0 from '/Users/danielblignaut/conductor/repo/soma/philadelphia/js/examples/insurance-claim-bot/functions/approveClaim.ts';
import agent0 from '/Users/danielblignaut/conductor/repo/soma/philadelphia/js/examples/insurance-claim-bot/agents/index.ts';

	console.log("SDK server starting...");

// Start gRPC server (don't await - it runs forever)
const socketPath = process.env.SOMA_SERVER_SOCK || '/tmp/soma-sdk.sock';
const projectDir = process.cwd();
startGrpcServer(socketPath, projectDir).catch(err => {
  console.error('gRPC server error:', err);
  process.exit(1);
});

// Wait a bit for server to initialize
await new Promise(resolve => setTimeout(resolve, 100));
console.log(`gRPC server started on ${socketPath}`);

// Register secret handler to inject secrets into process.env
console.log('[INFO] Registering secret handler...');
setSecretHandler(async (err, secrets) => {
  if (err) {
    console.error('Error in secret handler:', err);
    const error: CallbackError = {
      message: err.message,
    }
    const res: SetSecretsResponse = {
      error
    }
    return res;
  }
  const secretKeys = secrets.map(s => s.key);
  console.log(`[INFO] Secret handler invoked with ${secrets.length} secrets: ${secretKeys.join(', ')}`);
  for (const secret of secrets) {
    process.env[secret.key] = secret.value;
    console.log(`[INFO] Set process.env.${secret.key}`);
  }
  const message = `Injected ${secrets.length} secrets into process.env`;
  console.log(`[INFO] Secret handler completed: ${message}`);
  const data: SetSecretsSuccess = {
    message,
  }
  const res: SetSecretsResponse = {
    data,
  }
  return res;
});
console.log('[INFO] Secret handler registered successfully');

// Register environment variable handler to inject environment variables into process.env
console.log('[INFO] Registering environment variable handler...');
setEnvironmentVariableHandler(async (err, envVars) => {
  if (err) {
    console.error('Error in environment variable handler:', err);
    const error: CallbackError = {
      message: err.message,
    }
    const res: SetEnvironmentVariablesResponse = {
      error
    }
    return res;
  }
  const envVarKeys = envVars.map(e => e.key);
  console.log(`[INFO] Environment variable handler invoked with ${envVars.length} environment variables: ${envVarKeys.join(', ')}`);
  for (const envVar of envVars) {
    process.env[envVar.key] = envVar.value;
    console.log(`[INFO] Set process.env.${envVar.key}`);
  }
  const message = `Injected ${envVars.length} environment variables into process.env`;
  console.log(`[INFO] Environment variable handler completed: ${message}`);
  const data: SetEnvironmentVariablesSuccess = {
    message,
  }
  const res: SetEnvironmentVariablesResponse = {
    data,
  }
  return res;
});
console.log('[INFO] Environment variable handler registered successfully');

// Register unset secret handler to remove secrets from process.env
console.log('[INFO] Registering unset secret handler...');
setUnsetSecretHandler(async (err, key) => {
  if (err) {
    console.error('Error in unset secret handler:', err);
    const error: CallbackError = {
      message: err.message,
    }
    const res: UnsetSecretResponse = {
      error
    }
    return res;
  }
  console.log(`[INFO] Unset secret handler invoked with key: ${key}`);
  delete process.env[key];
  console.log(`[INFO] Removed process.env.${key}`);
  const message = `Removed secret '${key}' from process.env`;
  console.log(`[INFO] Unset secret handler completed: ${message}`);
  const data: UnsetSecretSuccess = {
    message,
  }
  const res: UnsetSecretResponse = {
    data,
  }
  return res;
});
console.log('[INFO] Unset secret handler registered successfully');

// Register unset environment variable handler to remove environment variables from process.env
console.log('[INFO] Registering unset environment variable handler...');
setUnsetEnvironmentVariableHandler(async (err, key) => {
  if (err) {
    console.error('Error in unset environment variable handler:', err);
    const error: CallbackError = {
      message: err.message,
    }
    const res: UnsetEnvironmentVariableResponse = {
      error
    }
    return res;
  }
  console.log(`[INFO] Unset environment variable handler invoked with key: ${key}`);
  delete process.env[key];
  console.log(`[INFO] Removed process.env.${key}`);
  const message = `Removed environment variable '${key}' from process.env`;
  console.log(`[INFO] Unset environment variable handler completed: ${message}`);
  const data: UnsetEnvironmentVariableSuccess = {
    message,
  }
  const res: UnsetEnvironmentVariableResponse = {
    data,
  }
  return res;
});
console.log('[INFO] Unset environment variable handler registered successfully');

// Register all providers and functions

  // Register function: approveClaim
  {
    const fn = await func0;
    if (fn.providerController) {
      addProvider(fn.providerController);
    }
    if (fn.functionController && fn.providerController && fn.handler) {
      const providerTypeId = fn.providerController.typeId || fn.providerController.typeId;
      const functionMetadata = {
        name: fn.functionController.name,
        description: fn.functionController.description,
        parameters: fn.functionController.parameters,
        output: fn.functionController.output,
      };

      // Create invoke callback that calls the handler
      const invokeCallback = async (err, req) => {
        if (err) {
          return { error: err.message };
        }

        try {
          // Parse the parameters and call the handler
          const params = JSON.parse(req.parameters);
          console.log(params);
          const result = await fn.handler(params);
          console.log(result);
          return { data: JSON.stringify(result) };
        } catch (error) {
          console.error(error);
          return { error: error.message || String(error) };
        }
      };

      addFunction(providerTypeId, functionMetadata, invokeCallback);
    }
  }

// Register all agents

  // Register agent: index
  {
    const agent = await agent0;
    if (agent.agentId && agent.projectId && agent.name && agent.description) {
      addAgent({
        id: agent.agentId,
        projectId: agent.projectId,
        name: agent.name,
        description: agent.description,
      });
    }
  }

console.log("SDK server ready!");


import { HandlerParams, SomaAgent } from "@trysoma/sdk/agent";
import { V1Api as SomaV1Api, Configuration as SomaConfiguration } from '@trysoma/api-client';
import * as net from 'net';

interface RestateInput {
  taskId: string;
  contextId: string;
}

type RestateHandler = (ctx: restate.ObjectContext, input: RestateInput) => Promise<void>;
type SomaHandler<T> = (params: HandlerParams) => Promise<void>;
const wrapHandler = <T>(handler: SomaHandler<T>, agent: SomaAgent): RestateHandler => {
  return async (ctx, input) => {
    const soma = new SomaV1Api(new SomaConfiguration({
      basePath: process.env.SOMA_SERVER_BASE_URL || 'http://localhost:3000',
    }));
    await handler({
      ctx,
      soma,
      taskId: input.taskId,
      contextId: input.contextId,
    });
  };
}

const restateServicePort = process.env.RESTATE_SERVICE_PORT;
if (!restateServicePort) {
  throw new Error('RESTATE_SERVICE_PORT environment variable is not set');
}
const restatePort = parseInt(restateServicePort);
console.log(`Starting Restate server on port ${restatePort}...`);

// Wait for port to become available (in case previous instance is shutting down from HMR)
const checkPortAvailable = (port: number): Promise<boolean> => {
  return new Promise((resolve) => {
    const server = net.createServer();
    server.listen(port, () => {
      server.once('close', () => resolve(true));
      server.close();
    });
    server.on('error', () => resolve(false));
  });
};

// Wait for port to become available with exponential backoff
const waitForPortAvailable = async (port: number, maxWaitSeconds: number = 30): Promise<void> => {
  const startTime = Date.now();
  let attempt = 0;
  
  while (Date.now() - startTime < maxWaitSeconds * 1000) {
    const available = await checkPortAvailable(port);
    if (available) {
      if (attempt > 0) {
        console.log(`Port ${port} is now available after waiting for previous instance to shut down.`);
      }
      return;
    }
    
    // Exponential backoff: 100ms, 200ms, 400ms, 800ms, then cap at 1s
    const delayMs = Math.min(100 * Math.pow(2, attempt), 1000);
    await new Promise(resolve => setTimeout(resolve, delayMs));
    attempt++;
  }
  
  throw new Error(`Port ${port} did not become available within ${maxWaitSeconds} seconds. Please check if another process is using the port.`);
};

// Wait for port to be available before starting (handles HMR shutdown gracefully)
try {
  await waitForPortAvailable(restatePort);
} catch (error) {
  console.error(error.message);
  process.exit(1);
}

// Create HTTP/2 server with Restate endpoint handler
const http2Handler = restate.createEndpointHandler({
  services: [
    restate.object({
      name: `${agent0.projectId}.${agent0.agentId}`,
      handlers: Object.fromEntries([
        ['entrypoint', wrapHandler(agent0.entrypoint, agent0)],
        // ...Object.entries(agent0.handlers).map(([key, value]) => [key, wrapHandler(value, agent0)])
      ]),
    })
  ],
});
const httpServer = http2.createServer(http2Handler);

// Handle graceful shutdown
let isShuttingDown = false;
const shutdown = async () => {
  if (isShuttingDown) return;
  isShuttingDown = true;
  console.log('\nShutting down Restate server...');
  return new Promise<void>((resolve) => {
    httpServer.close(() => {
      console.log('Restate server closed');
      resolve();
    });
    // Force close after 5 seconds if graceful shutdown doesn't complete
    setTimeout(() => {
      console.log('Forcing Restate server shutdown...');
      resolve();
    }, 5000);
  });
};

process.on('SIGINT', async () => {
  await shutdown();
  process.exit(0);
});
process.on('SIGTERM', async () => {
  await shutdown();
  process.exit(0);
});
process.on('SIGHUP', async () => {
  await shutdown();
  process.exit(0);
});

// Handle server errors (must be set before listen)
httpServer.on('error', (err: Error) => {
  if ((err as any).code === 'EADDRINUSE') {
    console.error(`Port ${restatePort} is already in use. Please stop the existing server or use a different port.`);
  } else {
    console.error('Restate server error:', err);
  }
  process.exit(1);
});

// Start the server
httpServer.listen(restatePort, async () => {
  console.log(`Restate server listening on port ${restatePort}`);

  // Trigger resync with API server to sync providers, agents, secrets, and env vars
  // This must happen AFTER the Restate server is listening, so Restate can verify the deployment
  // Retry with backoff since the API server may not be ready yet
  const maxRetries = 10;
  const baseDelayMs = 500;
  let resyncSuccess = false;

  for (let attempt = 1; attempt <= maxRetries && !resyncSuccess; attempt++) {
    console.log(`Triggering resync with API server (attempt ${attempt}/${maxRetries})...`);
    try {
      await resyncSdk();
      console.log("Resync with API server completed successfully");
      resyncSuccess = true;
    } catch (error) {
      if (attempt < maxRetries) {
        const delayMs = baseDelayMs * attempt;
        console.log(`Resync failed, retrying in ${delayMs}ms...`);
        await new Promise(resolve => setTimeout(resolve, delayMs));
      } else {
        console.error("Failed to resync with API server after all retries:", error);
        // Don't exit - secrets/env vars will be synced when the API server connects
      }
    }
  }
});

// Keep the process alive
await new Promise(() => {});
