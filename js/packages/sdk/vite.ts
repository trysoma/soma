import { type ChildProcess, spawn } from "node:child_process";
import {
	existsSync,
	mkdirSync,
	readdirSync,
	renameSync,
	rmSync,
	statSync,
	writeFileSync,
} from "node:fs";
import { join, parse, relative, resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { defineConfig, type Plugin } from "vite";

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
	for (const [name, path] of Object.entries(functionFiles)) {
		const varName = `func${funcIndex++}`;
		const importPath = isDev ? path : `./functions/${name}.js`;
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
  }`);
	}

	// Generate imports and registrations for agents
	let agentIndex = 0;
	for (const [name, path] of Object.entries(agentFiles)) {
		const varName = `agent${agentIndex++}`;
		const importPath = isDev ? path : `./agents/${name}.js`;
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

	return `// Auto-generated standalone server
import { addFunction, addProvider, addAgent, startGrpcServer } from '@trysoma/sdk';
import * as restate from '@restatedev/restate-sdk';
import * as http2 from 'http2';

${functionImports.join("\n")}
${agentImports.join("\n")}

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
console.log(\`gRPC server started on \${socketPath}\`);

// Register all providers and functions
${functionRegistrations.join("\n")}

// Register all agents
${agentRegistrations.join("\n")}

console.log("SDK server ready!");
${
	hasAgents
		? `
import { HandlerParams, SomaAgent } from "@trysoma/sdk/agent";
import { Configuration as BridgeConfiguration } from '@trysoma/sdk/bridge';
import { DefaultApi, Configuration as SomaConfiguration } from '@trysoma/api-client';
import * as net from 'net';

interface RestateInput {
  taskId: string;
  contextId: string;
}

type RestateHandler = (ctx: restate.ObjectContext, input: RestateInput) => Promise<void>;
type SomaHandler<T> = (params: HandlerParams<T>) => Promise<void>;
const wrapHandler = <T>(handler: SomaHandler<T>, agent: SomaAgent<T>): RestateHandler => {
  return async (ctx, input) => {
    const soma = new DefaultApi(new SomaConfiguration({
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

const restatePort = parseInt(process.env.RESTATE_RUNTIME_PORT || process.env.RESTATE_PORT || '9080');
console.log(\`Starting Restate server on port \${restatePort}...\`);

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
        console.log(\`Port \${port} is now available after waiting for previous instance to shut down.\`);
      }
      return;
    }
    
    // Exponential backoff: 100ms, 200ms, 400ms, 800ms, then cap at 1s
    const delayMs = Math.min(100 * Math.pow(2, attempt), 1000);
    await new Promise(resolve => setTimeout(resolve, delayMs));
    attempt++;
  }
  
  throw new Error(\`Port \${port} did not become available within \${maxWaitSeconds} seconds. Please check if another process is using the port.\`);
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
${restateServices.join(",\n")}
  ],
});
const httpServer = http2.createServer(http2Handler);

// Handle graceful shutdown
let isShuttingDown = false;
const shutdown = async () => {
  if (isShuttingDown) return;
  isShuttingDown = true;
  console.log('\\nShutting down Restate server...');
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
  // Don't call process.exit() - let the parent process manager handle shutdown
});
process.on('SIGTERM', async () => {
  await shutdown();
  // Don't call process.exit() - let the parent process manager handle shutdown
});
process.on('SIGHUP', async () => {
  await shutdown();
  // Don't call process.exit() - let the parent process manager handle shutdown
});

// Handle server errors (must be set before listen)
httpServer.on('error', (err: Error) => {
  if ((err as any).code === 'EADDRINUSE') {
    console.error(\`Port \${restatePort} is already in use. Please stop the existing server or use a different port.\`);
  } else {
    console.error('Restate server error:', err);
  }
  process.exit(1);
});

// Start the server
httpServer.listen(restatePort, () => {
  console.log(\`Restate server listening on port \${restatePort}\`);
});
`
		: `
// No agents defined, skipping Restate server startup
// Handle graceful shutdown for gRPC server only
process.on('SIGINT', () => {
  console.log('\\nShutting down...');
  // Don't call process.exit() - let the parent process manager handle shutdown
});
process.on('SIGTERM', () => {
  console.log('\\nShutting down...');
  // Don't call process.exit() - let the parent process manager handle shutdown
});
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
	const standaloneFilePath = resolve(baseDir, ".soma/standalone.ts");

	function regenerateStandalone() {
		const content = generateStandaloneServer(baseDir, true);
		writeFileSync(standaloneFilePath, content);
		console.log("Generated standalone.ts");
	}

	function restartServer() {
		if (serverProcess) {
			console.log("Restarting SDK server...");
			serverProcess.kill();
			serverProcess = null;
		}

		// Start the server in a separate Node process
		// We use tsx to transpile TypeScript on-the-fly since the standalone.ts
		// imports source .ts files directly (functions/agents)
		serverProcess = spawn("npx", ["tsx", standaloneFilePath], {
			stdio: "inherit",
			cwd: baseDir,
			shell: true,
		});

		serverProcess.on("error", (err) => {
			console.error("Failed to start SDK server:", err);
		});

		serverProcess.on("exit", (code) => {
			if (code !== null && code !== 0) {
				console.error(`SDK server exited with code ${code}`);
			}
		});
	}

	return {
		name: "soma-standalone-server",

		configureServer(devServer) {
			// Generate standalone file on startup
			regenerateStandalone();

			// Watch for changes in functions and agents directories
			const functionsDir = resolve(baseDir, "functions");
			const agentsDir = resolve(baseDir, "agents");

			devServer.watcher.add([functionsDir, agentsDir]);

			devServer.watcher.on("add", (file) => {
				if (file.includes("/functions/") || file.includes("/agents/")) {
					console.log(`New file detected: ${file}`);
					regenerateStandalone();
					if (serverProcess) {
						restartServer();
					}
				}
			});

			devServer.watcher.on("unlink", (file) => {
				if (file.includes("/functions/") || file.includes("/agents/")) {
					console.log(`File removed: ${file}`);
					regenerateStandalone();
					if (serverProcess) {
						restartServer();
					}
				}
			});

			devServer.watcher.on("change", (file) => {
				if (file.includes("/functions/") || file.includes("/agents/")) {
					console.log(`File changed: ${file}`);
					regenerateStandalone();
					if (serverProcess) {
						restartServer();
					}
				}
			});

			// Start server after Vite is ready
			devServer.httpServer?.once("listening", () => {
				setTimeout(() => {
					console.log("\nStarting SDK server...");
					restartServer();
				}, 1000);
			});
		},

		buildEnd() {
			// Kill server process when build ends
			if (serverProcess) {
				serverProcess.kill();
				serverProcess = null;
			}
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

		async writeBundle(options, bundle) {
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
			manifest.provider_controllers = Array.from(
				seenProviderControllers.values(),
			);

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
					treeshake: {
						preset: "recommended",
						moduleSideEffects: false,
					},
				},
			},
			resolve: {
				conditions: ["node", "import"],
			},
		};
	});
}
