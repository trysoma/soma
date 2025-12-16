/// <reference types="node" />
import { type ChildProcess, spawn } from "node:child_process";
import {
	existsSync,
	mkdirSync,
	readdirSync,
	statSync,
	writeFileSync,
} from "node:fs";
import { join, parse, relative, resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import type { NormalizedOutputOptions, OutputBundle } from "rollup";
import { defineConfig, type Plugin, type ViteDevServer } from "vite";

/**
 * Manifest structure types
 */
interface FunctionController {
	name?: string;
	description?: string;
	parameters?: unknown;
	output?: unknown;
	file: string;
	error?: string;
}

interface ProviderController {
	typeId?: string;
	type_id?: string;
	name?: string;
	[key: string]: unknown;
}

interface AgentMetadata {
	file: string;
	agentId?: string;
	projectId?: string;
	name?: string;
	description?: string;
	handlers?: string[];
	error?: string;
	[key: string]: unknown;
}

interface Manifest {
	function_controllers: FunctionController[];
	provider_controllers: ProviderController[];
	agents: AgentMetadata[];
}

/**
 * Safely extract error message from unknown error type
 */
function getErrorMessage(error: unknown): string {
	if (error instanceof Error) {
		return error.message;
	}
	return String(error);
}

/**
 * Recursively find all .ts files in a directory
 */
function findTsFiles(
	dir: string,
	baseDir: string = dir,
): Record<string, string> {
	const entries: Record<string, string> = {};

	if (!existsSync(dir)) {
		return entries;
	}

	const files = readdirSync(dir);

	for (const file of files) {
		const fullPath = join(dir, file);
		const stat = statSync(fullPath);

		if (stat.isDirectory()) {
			// Recursively search subdirectories
			Object.assign(entries, findTsFiles(fullPath, baseDir));
		} else if (file.endsWith(".ts") && !file.endsWith(".d.ts")) {
			// Get relative path from base directory
			const relativePath = relative(baseDir, fullPath);
			// Create entry name by removing .ts extension and replacing path separators
			const { dir, name } = parse(relativePath);
			const entryName = dir ? `${dir}/${name}`.replace(/\\/g, "/") : name;
			entries[entryName] = fullPath;
		}
	}

	return entries;
}

/**
 * Build entry points for functions and agents
 */
function buildSomaEntries(baseDir: string) {
	const functionsDir = resolve(baseDir, "functions");
	const agentsDir = resolve(baseDir, "agents");

	const functionEntries = findTsFiles(functionsDir);
	const agentEntries = findTsFiles(agentsDir);

	// Prefix entries to maintain directory structure
	const entries: Record<string, string> = {};

	for (const [name, path] of Object.entries(functionEntries)) {
		entries[`functions/${name}`] = path;
	}

	for (const [name, path] of Object.entries(agentEntries)) {
		entries[`agents/${name}`] = path;
	}

	return entries;
}

/**
 * Deep equality check for provider controllers, excluding functions
 */
function areProviderControllersEqual(
	a: ProviderController,
	b: ProviderController,
): boolean {
	if (a === b) return true;
	if (!a || !b) return false;

	// Create copies without the functions property and invoke property
	const aCopy = JSON.parse(
		JSON.stringify(a, (key, value) => {
			if (key === "invoke") return undefined;
			return value;
		}),
	);
	const bCopy = JSON.parse(
		JSON.stringify(b, (key, value) => {
			if (key === "invoke") return undefined;
			return value;
		}),
	);

	return isDeepStrictEqual(aCopy, bCopy);
}

/**
 * Generate standalone server entrypoint
 */
function generateStandaloneServer(
	baseDir: string,
	isDev: boolean = false,
): string {
	const functionsDir = resolve(baseDir, "functions");
	const agentsDir = resolve(baseDir, "agents");

	const functionFiles = findTsFiles(functionsDir);
	const agentFiles = findTsFiles(agentsDir);

	const functionImports: string[] = [];
	const functionRegistrations: string[] = [];
	const agentImports: string[] = [];
	const agentRegistrations: string[] = [];

	// Generate imports and registrations for functions
	let funcIndex = 0;
	for (const [name, _path] of Object.entries(functionFiles)) {
		const varName = `func${funcIndex++}`;
		// Use relative paths for both dev and prod to avoid absolute path issues in CI
		const importPath = isDev
			? `../functions/${name}`
			: `./functions/${name}.js`;
		functionImports.push(`import ${varName} from '${importPath}';`);
		functionRegistrations.push(`
  // Register function: ${name}
  {
    const fn = await ${varName};
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
      const invokeCallback = async (err: Error | null, req: InvokeFunctionRequest): Promise<InvokeFunctionResponse> => {
        if (err) {
          const callbackError: CallbackError = {
            message: err.message,
          };
          return { error: callbackError };
        }

        try {
          // Parse the parameters and call the handler
          const params = JSON.parse(req.parameters);
          console.log(params);
          const result = await fn.handler(params);
          console.log(result);
          return { data: JSON.stringify(result) };
        } catch (error: unknown) {
          console.error(error);
          const errorMessage = error instanceof Error ? error.message : String(error);
          const callbackError: CallbackError = {
            message: errorMessage,
          };
          return { error: callbackError };
        }
      };

      addFunction(providerTypeId, functionMetadata, invokeCallback);
    }
  }`);
	}

	// Generate imports and registrations for agents
	let agentIndex = 0;
	for (const [name, _path] of Object.entries(agentFiles)) {
		const varName = `agent${agentIndex++}`;
		// Use relative paths for both dev and prod to avoid absolute path issues in CI
		const importPath = isDev ? `../agents/${name}` : `./agents/${name}.js`;
		agentImports.push(`import ${varName} from '${importPath}';`);
		agentRegistrations.push(`
  // Register agent: ${name}
  {
    const agent = await ${varName};
    if (agent.agentId && agent.projectId && agent.name && agent.description) {
      addAgent({
        id: agent.agentId,
        projectId: agent.projectId,
        name: agent.name,
        description: agent.description,
      });
    }
  }`);
	}

	// Generate the Restate agent services array
	const restateServices: string[] = [];
	for (let i = 0; i < agentIndex; i++) {
		const varName = `agent${i}`;
		restateServices.push(`    restate.object({
      name: \`\${${varName}.projectId}.\${${varName}.agentId}\`,
      handlers: Object.fromEntries([
        ['entrypoint', wrapHandler(${varName}.entrypoint, ${varName})],
        // ...Object.entries(${varName}.handlers).map(([key, value]) => [key, wrapHandler(value, ${varName})])
      ]),
    })`);
	}

	const hasAgents = agentIndex > 0;

	return `/// <reference types="node" />
// Auto-generated standalone server
import { addFunction, addProvider, addAgent, startGrpcServer, killGrpcService, setSecretHandler, setEnvironmentVariableHandler, setUnsetSecretHandler, setUnsetEnvironmentVariableHandler, resyncSdk } from '@trysoma/sdk';
import type { Secret, SetSecretsResponse, SetSecretsSuccess, CallbackError, EnvironmentVariable, SetEnvironmentVariablesResponse, SetEnvironmentVariablesSuccess, UnsetSecretResponse, UnsetSecretSuccess, UnsetEnvironmentVariableResponse, UnsetEnvironmentVariableSuccess, InvokeFunctionRequest, InvokeFunctionResponse } from '@trysoma/sdk';
import * as restate from '@restatedev/restate-sdk';
import * as http2 from 'http2';

${functionImports.join("\n")}
${agentImports.join("\n")}

console.debug("[SDK] Starting");

// Track shutdown state (declared early so error handlers can access it)
let isShuttingDown = false;

// Handle uncaught exceptions and unhandled rejections gracefully during shutdown
process.on('uncaughtException', (err: Error) => {
  if (!isShuttingDown) {
    console.error('[SDK] Uncaught exception:', err);
    process.exit(1);
  }
});

process.on('unhandledRejection', (reason: unknown) => {
  if (!isShuttingDown) {
    console.error('[SDK] Unhandled rejection:', reason);
    process.exit(1);
  }
});

// Start gRPC server (don't await - it runs forever)
const socketPath = process.env.SOMA_SERVER_SOCK || '/tmp/soma-sdk.sock';
const projectDir = process.cwd();
startGrpcServer(socketPath, projectDir).catch(err => {
  if (!isShuttingDown) {
    console.error('[SDK] gRPC server error:', err);
    process.exit(1);
  }
});

await new Promise(resolve => setTimeout(resolve, 100));
console.debug(\`[SDK] gRPC server started on \${socketPath}\`);

// Register secret handler
setSecretHandler(async (err, secrets) => {
  if (err) {
    console.error('[SDK] Secret handler error:', err);
    return { error: { message: err.message } };
  }
  console.debug(\`[SDK] Setting \${secrets.length} secrets\`);
  for (const secret of secrets) {
    process.env[secret.key] = secret.value;
  }
  return { data: { message: \`Injected \${secrets.length} secrets\` } };
});

// Register environment variable handler
setEnvironmentVariableHandler(async (err, envVars) => {
  if (err) {
    console.error('[SDK] Env var handler error:', err);
    return { error: { message: err.message } };
  }
  console.debug(\`[SDK] Setting \${envVars.length} env vars\`);
  for (const envVar of envVars) {
    process.env[envVar.key] = envVar.value;
  }
  return { data: { message: \`Injected \${envVars.length} env vars\` } };
});

// Register unset secret handler
setUnsetSecretHandler(async (err, key) => {
  if (err) {
    console.error('[SDK] Unset secret handler error:', err);
    return { error: { message: err.message } };
  }
  console.debug(\`[SDK] Unsetting secret \${key}\`);
  delete process.env[key];
  return { data: { message: \`Removed secret '\${key}'\` } };
});

// Register unset environment variable handler
setUnsetEnvironmentVariableHandler(async (err, key) => {
  if (err) {
    console.error('[SDK] Unset env var handler error:', err);
    return { error: { message: err.message } };
  }
  console.debug(\`[SDK] Unsetting env var \${key}\`);
  delete process.env[key];
  return { data: { message: \`Removed env var '\${key}'\` } };
});

// Register all providers and functions
${functionRegistrations.join("\n")}

// Register all agents
${agentRegistrations.join("\n")}

console.debug("[SDK] Ready");

${
	hasAgents
		? `
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
  throw new Error('RESTATE_SERVICE_PORT not set');
}
const restatePort = parseInt(restateServicePort);
console.debug(\`[SDK] Starting Restate on port \${restatePort}\`);

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

const waitForPortAvailable = async (port: number, maxWaitSeconds: number = 30): Promise<void> => {
  const startTime = Date.now();
  let attempt = 0;

  while (Date.now() - startTime < maxWaitSeconds * 1000) {
    const available = await checkPortAvailable(port);
    if (available) return;
    const delayMs = Math.min(100 * Math.pow(2, attempt), 1000);
    await new Promise(resolve => setTimeout(resolve, delayMs));
    attempt++;
  }

  throw new Error(\`Port \${port} not available within \${maxWaitSeconds}s\`);
};

try {
  await waitForPortAvailable(restatePort);
} catch (error: unknown) {
  if (isShuttingDown) process.exit(0);
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}

// Create HTTP/2 server with Restate endpoint handler
const http2Handler = restate.createEndpointHandler({
  services: [
${restateServices.join(",\n")}
  ],
});
const httpServer = http2.createServer(http2Handler);

const shutdown = async () => {
  if (isShuttingDown) return;
  isShuttingDown = true;
  console.debug('[SDK] Shutting down');

  try {
    killGrpcService();
  } catch (err) {
    console.error('[SDK] Error killing gRPC:', err);
  }

  // Force close HTTP server immediately - don't wait for graceful shutdown
  // This ensures the port is released even if the process is killed
  return new Promise<void>((resolve) => {
    let resolved = false;
    const doResolve = () => {
      if (!resolved) {
        resolved = true;
        resolve();
      }
    };

    // Close the server - this stops accepting new connections
    httpServer.close(() => {
      doResolve();
    });

    // Force resolve after 500ms - don't wait too long
    setTimeout(() => {
      doResolve();
    }, 500);
  });
};

// Handle shutdown signals - ensure shutdown completes quickly before exiting
// Use Promise.race to ensure we exit even if shutdown hangs
process.on('SIGINT', async () => {
  await Promise.race([
    shutdown(),
    new Promise(resolve => setTimeout(resolve, 2000)) // Max 2 seconds
  ]);
  process.exit(0);
});
process.on('SIGTERM', async () => {
  await Promise.race([
    shutdown(),
    new Promise(resolve => setTimeout(resolve, 2000)) // Max 2 seconds
  ]);
  process.exit(0);
});
process.on('SIGHUP', async () => {
  await Promise.race([
    shutdown(),
    new Promise(resolve => setTimeout(resolve, 2000)) // Max 2 seconds
  ]);
  process.exit(0);
});

httpServer.on('error', (err: NodeJS.ErrnoException) => {
  if (isShuttingDown) return;
  if (err.code === 'EADDRINUSE') {
    console.error(\`[SDK] Port \${restatePort} in use\`);
  } else {
    console.error('[SDK] Restate error:', err);
  }
  process.exit(1);
});

httpServer.listen(restatePort, async () => {
  console.debug(\`[SDK] Restate listening on port \${restatePort}\`);

  const maxRetries = 10;
  const baseDelayMs = 500;
  let resyncSuccess = false;

  for (let attempt = 1; attempt <= maxRetries && !resyncSuccess && !isShuttingDown; attempt++) {
    if (isShuttingDown) break;
    console.debug(\`[SDK] Resync attempt \${attempt}/\${maxRetries}\`);
    try {
      await resyncSdk();
      console.debug("[SDK] Resync completed");
      resyncSuccess = true;
    } catch (error) {
      if (attempt < maxRetries && !isShuttingDown) {
        const delayMs = baseDelayMs * attempt;
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(resolve, delayMs);
          const checkShutdown = setInterval(() => {
            if (isShuttingDown) {
              clearTimeout(timeout);
              clearInterval(checkShutdown);
              resolve();
            }
          }, 100);
          setTimeout(() => clearInterval(checkShutdown), delayMs);
        });
      } else if (!isShuttingDown) {
        console.error("[SDK] Resync failed after retries:", error);
      }
    }
  }
});
`
		: `
let isShuttingDownNoAgents = false;

process.on('uncaughtException', (err: Error) => {
  if (!isShuttingDownNoAgents) {
    console.error('[SDK] Uncaught exception:', err);
    process.exit(1);
  }
});

process.on('unhandledRejection', (reason: unknown) => {
  if (!isShuttingDownNoAgents) {
    console.error('[SDK] Unhandled rejection:', reason);
    process.exit(1);
  }
});

const shutdownNoAgents = () => {
  if (isShuttingDownNoAgents) return;
  isShuttingDownNoAgents = true;
  console.debug('[SDK] Shutting down');
  try {
    killGrpcService();
  } catch (err) {
    console.error('[SDK] Error killing gRPC:', err);
  }
  process.exit(0);
};
process.on('SIGINT', shutdownNoAgents);
process.on('SIGTERM', shutdownNoAgents);

const maxRetries = 10;
const baseDelayMs = 500;
let resyncSuccess = false;

for (let attempt = 1; attempt <= maxRetries && !resyncSuccess && !isShuttingDownNoAgents; attempt++) {
  if (isShuttingDownNoAgents) break;
  console.debug(\`[SDK] Resync attempt \${attempt}/\${maxRetries}\`);
  try {
    await resyncSdk();
    console.debug("[SDK] Resync completed");
    resyncSuccess = true;
  } catch (error) {
    if (attempt < maxRetries && !isShuttingDownNoAgents) {
      const delayMs = baseDelayMs * attempt;
      await new Promise<void>((resolve) => {
        const timeout = setTimeout(resolve, delayMs);
        const checkShutdown = setInterval(() => {
          if (isShuttingDownNoAgents) {
            clearTimeout(timeout);
            clearInterval(checkShutdown);
            resolve();
          }
        }, 100);
        setTimeout(() => clearInterval(checkShutdown), delayMs);
      });
    } else if (!isShuttingDownNoAgents) {
      console.error("[SDK] Resync failed after retries:", error);
    }
  }
}
`
}
// Keep the process alive
await new Promise(() => {});
`;
}

/**
 * Plugin to generate and run standalone server in dev mode
 * Note: Runs in a separate Node process to avoid conflicts with Vite's module system
 */
function standaloneServerPlugin(baseDir: string): Plugin {
	let serverProcess: ChildProcess | null = null;
	const standaloneFilePath = resolve(baseDir, "soma/standalone.ts");
	let watcherHandlers: Array<{
		event: string;
		handler: (file: string) => void;
	}> = [];
	let devServerRef: ViteDevServer | null = null;

	// Cleanup function to kill SDK server process and remove watchers
	const cleanup = () => {
		// Remove watcher handlers to prevent further events
		if (devServerRef) {
			for (const { event, handler } of watcherHandlers) {
				try {
					devServerRef.watcher.off(event, handler);
				} catch (_e) {
					// Ignore errors - watcher might already be closed
				}
			}
			watcherHandlers = [];
		}

		if (serverProcess) {
			console.log("Killing SDK server process...");
			const pid = serverProcess.pid;
			// Kill the entire process group to ensure child processes are killed
			// When using shell: true, the actual process is a child of the shell
			if (pid) {
				try {
					// Kill process group (negative PID kills the group)
					process.kill(-pid, "SIGTERM");
				} catch (_e) {
					// Try killing just the process if group kill fails
					try {
						serverProcess.kill("SIGTERM");
					} catch (_e2) {
						// Process might already be dead
					}
				}
				// Force kill immediately - don't wait, as parent may exit
				try {
					process.kill(-pid, "SIGKILL");
				} catch (_e) {
					try {
						serverProcess.kill("SIGKILL");
					} catch (_e2) {
						// Process might already be dead
					}
				}
			}
			serverProcess = null;
		}
	};

	function regenerateStandalone() {
		const content = generateStandaloneServer(baseDir, true);
		const somaDir = resolve(baseDir, "soma");
		if (!existsSync(somaDir)) {
			mkdirSync(somaDir, { recursive: true });
		}
		writeFileSync(standaloneFilePath, content);

		// Ensure bridge.ts exists (even if empty) so imports don't fail
		const bridgeFilePath = resolve(baseDir, "soma/bridge.ts");
		if (!existsSync(bridgeFilePath)) {
			const emptyBridgeContent = `// Auto-generated Bridge Client
// Do not edit this file manually

import type { ObjectContext } from '@restatedev/restate-sdk';

export interface BridgeConfig {
  SOMA_BASE_URL?: string;
}

interface InvokeResult<T> {
  type: 'success' | 'error';
  data?: T;
  error?: { message: string };
}

/**
 * Internal helper to invoke a bridge function
 */
async function _invokeBridgeFunction<TParams, TResult>(
  ctx: ObjectContext,
  providerName: string,
  accountName: string,
  functionName: string,
  providerInstanceId: string,
  functionControllerTypeId: string,
  params: TParams,
  baseUrl: string
): Promise<TResult> {
  ctx.console.log(\`Invoking \${providerName}.\${accountName}.\${functionName}\`);

  const result = await ctx.run(\`fetch-\${providerName}-\${functionName}\`, async () => {
    const response = await fetch(
      \`\${baseUrl}/api/bridge/v1/provider/\${providerInstanceId}/function/\${functionControllerTypeId}/invoke\`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ params }),
      }
    );

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(\`HTTP \${response.status}: \${errorText}\`);
    }

    const result: InvokeResult<TResult> = await response.json();

    if (result.type === 'error') {
      throw new Error(result.error?.message || 'Unknown error');
    }

    return result.data as TResult;
  });

  ctx.console.log(\`Completed \${providerName}.\${accountName}.\${functionName}\`);
  return result;
}

// Type definitions for all functions

export type Bridge = Record<string, never>;

export type BridgeDefinition = Bridge;

export function getBridge(_ctx: ObjectContext, config?: BridgeConfig): Bridge {
  const _baseUrl = config?.SOMA_BASE_URL || process.env.SOMA_SERVER_BASE_URL || 'http://localhost:3000';

  return {};
}
`;
			writeFileSync(bridgeFilePath, emptyBridgeContent);
		}

		console.log("Generated standalone.ts");
	}

	function restartServer() {
		if (serverProcess) {
			console.log("Restarting SDK server...");
			const pid = serverProcess.pid;
			// Kill the entire process group
			if (pid) {
				try {
					process.kill(-pid, "SIGTERM");
				} catch (_e) {
					try {
						serverProcess.kill("SIGTERM");
					} catch (_e2) {
						// Process might already be dead
					}
				}
				// Force kill after 500ms if still running
				setTimeout(() => {
					try {
						process.kill(-pid, "SIGKILL");
					} catch (_e) {
						// Process might already be dead
					}
				}, 500);
			}
			serverProcess = null;
		}

		// Ensure standalone.ts exists before starting server
		if (!existsSync(standaloneFilePath)) {
			console.log("standalone.ts not found, regenerating...");
			regenerateStandalone();
		}

		// Start the server in a separate Node process
		// We use tsx to transpile TypeScript on-the-fly since the standalone.ts
		// imports source .ts files directly (functions/agents)
		serverProcess = spawn("npx", ["tsx", standaloneFilePath], {
			stdio: "inherit",
			cwd: baseDir,
			shell: true,
			detached: true, // Create new process group so we can kill the entire tree
		});

		serverProcess.on("error", (err: Error) => {
			console.error("Failed to start SDK server:", err);
		});

		serverProcess.on("exit", (code: number | null) => {
			if (code !== null && code !== 0) {
				console.error(`SDK server exited with code ${code}`);
			}
		});
	}

	// Register process exit handlers to ensure cleanup on unexpected exits
	const exitHandler = () => {
		cleanup();
	};
	process.on("exit", exitHandler);
	process.on("SIGINT", exitHandler);
	process.on("SIGTERM", exitHandler);
	process.on("SIGHUP", exitHandler);

	return {
		name: "soma-standalone-server",

		// Generate standalone file BEFORE dev server starts to avoid stale execution
		buildStart() {
			console.log("Regenerating standalone.ts before dev server start...");
			regenerateStandalone();
		},

		configureServer(devServer: ViteDevServer) {
			// Store dev server reference for cleanup
			devServerRef = devServer;

			// Standalone was already generated in buildStart, but regenerate again
			// in case configureServer is called without buildStart (shouldn't happen, but just in case)
			regenerateStandalone();

			// Watch for changes in functions and agents directories
			const functionsDir = resolve(baseDir, "functions");
			const agentsDir = resolve(baseDir, "agents");

			devServer.watcher.add([functionsDir, agentsDir]);

			const onAdd = (file: string) => {
				if (file.includes("/functions/") || file.includes("/agents/")) {
					console.log(`New file detected: ${file}`);
					regenerateStandalone();
					if (serverProcess) {
						restartServer();
					}
				}
			};

			const onUnlink = (file: string) => {
				if (file.includes("/functions/") || file.includes("/agents/")) {
					console.log(`File removed: ${file}`);
					regenerateStandalone();
					if (serverProcess) {
						restartServer();
					}
				}
			};

			const onChange = (file: string) => {
				if (file.includes("/functions/") || file.includes("/agents/")) {
					console.log(`File changed: ${file}`);
					regenerateStandalone();
					if (serverProcess) {
						restartServer();
					}
				}
			};

			devServer.watcher.on("add", onAdd);
			devServer.watcher.on("unlink", onUnlink);
			devServer.watcher.on("change", onChange);

			// Store handlers for cleanup (using top-level watcherHandlers)
			watcherHandlers = [
				{ event: "add", handler: onAdd },
				{ event: "unlink", handler: onUnlink },
				{ event: "change", handler: onChange },
			];

			// Start server after Vite is ready
			devServer.httpServer?.once("listening", () => {
				setTimeout(() => {
					console.log("\nStarting SDK server...");
					restartServer();
				}, 1000);
			});

			// Cleanup SDK server when Vite dev server closes
			devServer.httpServer?.on("close", () => {
				cleanup();
			});

			// Also handle Vite's WebSocket close event
			devServer.ws.on("close", () => {
				cleanup();
			});
		},

		buildEnd() {
			// Kill server process when build ends
			cleanup();
		},

		closeBundle() {
			// Kill server process when bundle closes
			cleanup();
		},
	};
}

/**
 * Vite plugin to generate manifest.json with full controller information
 * Uses a separate loader script to safely import modules
 */
function generateManifestPlugin(baseDir: string): Plugin {
	return {
		name: "soma-manifest-generator",

		async writeBundle(options: NormalizedOutputOptions, bundle: OutputBundle) {
			const outDir = options.dir || ".soma/build/js";
			const manifest: Manifest = {
				function_controllers: [],
				provider_controllers: [],
				agents: [],
			};

			// Track provider controllers to deduplicate
			const seenProviderControllers = new Map<string, ProviderController>();

			// Collect all function and agent files
			const functionFiles: string[] = [];
			const agentFiles: string[] = [];

			for (const [fileName, chunk] of Object.entries(bundle)) {
				if (chunk.type !== "chunk") continue;

				if (fileName.startsWith("functions/")) {
					functionFiles.push(fileName);
				} else if (fileName.startsWith("agents/")) {
					agentFiles.push(fileName);
				}
			}

			// Load each function module to extract metadata
			for (const fileName of functionFiles) {
				try {
					// Use dynamic import with a fresh module cache to avoid side effects
					const modulePath = resolve(outDir, fileName);
					// Use file:// protocol and add cache busting
					const moduleUrl = `file://${modulePath}?t=${Date.now()}`;

					const module = await import(moduleUrl);
					// The default export might be a Promise (if using top-level await in source)
					let defaultExport = module.default;
					if (defaultExport instanceof Promise) {
						defaultExport = await defaultExport;
					}

					if (!defaultExport) {
						console.warn(`No default export found in ${fileName}`);
						continue;
					}

					// Extract function controller
					if (defaultExport.functionController) {
						manifest.function_controllers.push({
							...defaultExport.functionController,
							file: fileName,
						});
					} else {
						console.warn(`No functionController found in ${fileName}`);
					}

					// Extract and deduplicate provider controller
					if (defaultExport.providerController) {
						const providerController = defaultExport.providerController;
						let foundDuplicate = false;

						for (const [
							existingKey,
							existingProvider,
						] of seenProviderControllers) {
							if (
								areProviderControllersEqual(
									providerController,
									existingProvider,
								)
							) {
								foundDuplicate = true;
								break;
							} else if (existingKey === providerController.typeId) {
								throw new Error(
									`Duplicate provider controller typeId: ${providerController.typeId} in ${fileName}`,
								);
							}
						}

						if (!foundDuplicate) {
							// Remove non-serializable properties
							const serializableProvider = JSON.parse(
								JSON.stringify(providerController, (key, value) => {
									if (key === "invoke" || typeof value === "function")
										return undefined;
									return value;
								}),
							);

							const key =
								serializableProvider.type_id ||
								serializableProvider.typeId ||
								serializableProvider.name;
							seenProviderControllers.set(key, serializableProvider);
						}
					} else {
						console.warn(`No providerController found in ${fileName}`);
					}
				} catch (err) {
					const errorMessage = getErrorMessage(err);
					console.warn(`Could not load ${fileName}:`, errorMessage);
					// Still add a placeholder entry so we know the file exists
					manifest.function_controllers.push({
						file: fileName,
						error: errorMessage,
					});
				}
			}

			// Load each agent module to extract metadata
			for (const fileName of agentFiles) {
				try {
					// Use dynamic import with a fresh module cache to avoid side effects
					const modulePath = resolve(outDir, fileName);
					// Use file:// protocol and add cache busting
					const moduleUrl = `file://${modulePath}?t=${Date.now()}`;

					const module = await import(moduleUrl);
					// The default export might be a Promise (if using top-level await in source)
					let defaultExport = module.default;
					if (defaultExport instanceof Promise) {
						defaultExport = await defaultExport;
					}

					if (!defaultExport) {
						console.warn(`No default export found in ${fileName}`);
						continue;
					}

					// Extract agent metadata (all fields from createSomaAgent)
					const agentMetadata: AgentMetadata = {
						file: fileName,
					};

					// Copy all fields from the agent object
					for (const [key, value] of Object.entries(defaultExport)) {
						// Skip non-serializable properties
						if (typeof value === "function") continue;

						// For handlers, just include the handler names
						if (
							key === "handlers" &&
							typeof value === "object" &&
							value !== null
						) {
							agentMetadata.handlers = Object.keys(value);
						} else {
							agentMetadata[key] = value;
						}
					}

					manifest.agents.push(agentMetadata);
				} catch (err) {
					const errorMessage = getErrorMessage(err);
					console.warn(`Could not load ${fileName}:`, errorMessage);
					// Still add a placeholder entry so we know the file exists
					manifest.agents.push({
						file: fileName,
						error: errorMessage,
					});
				}
			}

			// Convert provider controllers map to array
			manifest.provider_controllers = [...seenProviderControllers.values()];

			// Write manifest.json
			const manifestPath = resolve(outDir, "manifest.json");
			writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));

			// Generate standalone.js for production
			const standaloneContent = generateStandaloneServer(baseDir, false);
			const standalonePath = resolve(outDir, "standalone.js");
			writeFileSync(standalonePath, standaloneContent);

			console.log(`\nGenerated manifest.json with:`);
			console.log(
				`  - ${manifest.function_controllers.length} function controllers`,
			);
			console.log(
				`  - ${manifest.provider_controllers.length} provider controllers`,
			);
			console.log(`  - ${manifest.agents.length} agents`);
			console.log(`Generated standalone.js`);
		},
	};
}

/**
 * Create Soma Vite config
 * @param baseDir - The base directory of the project (usually __dirname from vite.config.ts)
 */
export function createSomaViteConfig(baseDir: string) {
	return defineConfig(({ command }) => {
		const isDev = command === "serve";

		return {
			plugins: [
				isDev && standaloneServerPlugin(baseDir),
				generateManifestPlugin(baseDir),
			].filter(Boolean),
			build: {
				target: "node18",
				outDir: ".soma/build/js",
				emptyOutDir: true,
				minify: false,
				ssr: true, // Enable SSR mode for Node.js builds
				lib: {
					entry: buildSomaEntries(baseDir),
					formats: ["es"],
				},
				rollupOptions: {
					external: [
						// Don't bundle Node.js built-ins
						/^node:/,
						"fs",
						"path",
						"url",
						"stream",
						"util",
						"events",
						"buffer",
						"crypto",
						"http",
						"https",
						"zlib",
						"net",
						"tls",
						"os",
						"process",
						"child_process",
						"assert",
						"dns",
						"dgram",
						"readline",
						"repl",
						"tty",
						"v8",
						"vm",
						"worker_threads",
					],
					output: {
						entryFileNames: "[name].js",
						chunkFileNames: "chunks/[name]-[hash].js",
						// Preserve directory structure
						preserveModules: false,
						manualChunks: undefined,
					},
					treeshake: "recommended",
				},
			},
			resolve: {
				conditions: ["node", "import"],
			},
		};
	});
}
