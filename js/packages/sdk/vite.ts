import { defineConfig, Plugin, ViteDevServer } from 'vite'
import { resolve } from 'node:path'
import { readdirSync, statSync, existsSync, writeFileSync, mkdirSync } from 'node:fs'
import { join, relative, parse } from 'node:path'
import { isDeepStrictEqual } from 'node:util'
import { spawn, ChildProcess } from 'node:child_process'

/**
 * Manifest structure types
 */
interface FunctionController {
  name?: string
  description?: string
  parameters?: unknown
  output?: unknown
  file: string
  error?: string
}

interface ProviderController {
  typeId?: string
  type_id?: string
  name?: string
  [key: string]: unknown
}

interface AgentMetadata {
  file: string
  agentId?: string
  projectId?: string
  name?: string
  description?: string
  handlers?: string[]
  generatedBridgeClient?: unknown
  error?: string
  [key: string]: unknown
}

interface Manifest {
  function_controllers: FunctionController[]
  provider_controllers: ProviderController[]
  agents: AgentMetadata[]
}

/**
 * Safely extract error message from unknown error type
 */
function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }
  return String(error)
}

/**
 * Recursively find all .ts files in a directory
 */
function findTsFiles(dir: string, baseDir: string = dir): Record<string, string> {
  const entries: Record<string, string> = {}

  if (!existsSync(dir)) {
    return entries
  }

  const files = readdirSync(dir)

  for (const file of files) {
    const fullPath = join(dir, file)
    const stat = statSync(fullPath)

    if (stat.isDirectory()) {
      // Recursively search subdirectories
      Object.assign(entries, findTsFiles(fullPath, baseDir))
    } else if (file.endsWith('.ts') && !file.endsWith('.d.ts')) {
      // Get relative path from base directory
      const relativePath = relative(baseDir, fullPath)
      // Create entry name by removing .ts extension and replacing path separators
      const { dir, name } = parse(relativePath)
      const entryName = dir ? `${dir}/${name}`.replace(/\\/g, '/') : name
      entries[entryName] = fullPath
    }
  }

  return entries
}

/**
 * Build entry points for functions and agents
 */
function buildSomaEntries(baseDir: string) {
  const functionsDir = resolve(baseDir, 'functions')
  const agentsDir = resolve(baseDir, 'agents')

  const functionEntries = findTsFiles(functionsDir)
  const agentEntries = findTsFiles(agentsDir)

  // Prefix entries to maintain directory structure
  const entries: Record<string, string> = {}

  for (const [name, path] of Object.entries(functionEntries)) {
    entries[`functions/${name}`] = path
  }

  for (const [name, path] of Object.entries(agentEntries)) {
    entries[`agents/${name}`] = path
  }

  return entries
}

/**
 * Deep equality check for provider controllers, excluding functions
 */
function areProviderControllersEqual(a: any, b: any): boolean {
  if (a === b) return true
  if (!a || !b) return false

  // Create copies without the functions property and invoke property
  const aCopy = JSON.parse(JSON.stringify(a, (key, value) => {
    if (key === 'invoke') return undefined
    return value
  }))
  const bCopy = JSON.parse(JSON.stringify(b, (key, value) => {
    if (key === 'invoke') return undefined
    return value
  }))

  return isDeepStrictEqual(aCopy, bCopy)
}

/**
 * Generate standalone server entrypoint
 */
function generateStandaloneServer(baseDir: string, isDev: boolean = false): string {
  const functionsDir = resolve(baseDir, 'functions')
  const agentsDir = resolve(baseDir, 'agents')

  const functionFiles = findTsFiles(functionsDir)
  const agentFiles = findTsFiles(agentsDir)

  const functionImports: string[] = []
  const functionRegistrations: string[] = []
  const agentImports: string[] = []
  const agentRegistrations: string[] = []

  // Generate imports and registrations for functions
  let funcIndex = 0
  for (const [name, path] of Object.entries(functionFiles)) {
    const varName = `func${funcIndex++}`
    const importPath = isDev ? path : `./functions/${name}.js`
    functionImports.push(`import ${varName} from '${importPath}';`)
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
  }`)
  }

  // Generate imports for agents (TODO: registration)
  let agentIndex = 0
  for (const [name, path] of Object.entries(agentFiles)) {
    const varName = `agent${agentIndex++}`
    const importPath = isDev ? path : `./agents/${name}.js`
    agentImports.push(`import ${varName} from '${importPath}';`)
    agentRegistrations.push(`
  // TODO: Register agent: ${name}
  // addAgent(await ${varName});`)
  }

  // Generate the Restate agent services array
  const restateServices: string[] = []
  for (let i = 0; i < agentIndex; i++) {
    const varName = `agent${i}`
    restateServices.push(`    restate.object({
      name: ${varName}.agentId,
      handlers: Object.fromEntries(Object.entries(${varName}.handlers).map(([key, value]) => [key, wrapHandler(value, ${varName})])),
    })`)
  }

  return `// Auto-generated standalone server
import { addFunction, addProvider, startGrpcServer } from '@soma/sdk';
import * as restate from '@restatedev/restate-sdk';

${functionImports.join('\n')}
${agentImports.join('\n')}

console.log("SDK server starting...");

// Start gRPC server (don't await - it runs forever)
const socketPath = process.env.SOMA_SERVER_SOCK || '/tmp/soma-sdk.sock';
startGrpcServer(socketPath).catch(err => {
  console.error('gRPC server error:', err);
  process.exit(1);
});

// Wait a bit for server to initialize
await new Promise(resolve => setTimeout(resolve, 100));
console.log(\`gRPC server started on \${socketPath}\`);

// Register all providers and functions
${functionRegistrations.join('\n')}

// Register all agents
${agentRegistrations.join('\n')}

console.log("SDK server ready!");

import { HandlerParams } from "@soma/sdk/agent";
import { Configuration as BridgeConfiguration } from '@soma/sdk/bridge';
import { DefaultApi, Configuration as SomaConfiguration } from '@soma/api-client';

interface RestateInput {
  taskId: string;
  contextId: string;
}

type RestateHandler = (ctx: restate.ObjectContext, input: RestateInput) => Promise<void>;
type SomaHandler<T> = (params: HandlerParams<T>) => Promise<void>;
const wrapHandler = (handler: SomaHandler<any>, agent: any): RestateHandler => {
  return async (ctx, input) => {
    const bridge = agent.generatedBridgeClient(process.env.SOMA_SERVER_BASE_URL || 'http://localhost:3000');
    const soma = new DefaultApi(new SomaConfiguration({
      basePath: process.env.SOMA_SERVER_BASE_URL || 'http://localhost:3000',
    }));
    await handler({
      ctx,
      bridge,
      soma,
      taskId: input.taskId,
      contextId: input.contextId,
    });
  };
}

await restate.serve({
  services: [
${restateServices.join(',\n')}
  ]
});

// Keep the process alive
await new Promise(() => {});
`
}

/**
 * Plugin to generate Bridge client from OpenAPI spec
 * Runs openapi-generator-cli before Vite dev server starts
 */
function bridgeClientPlugin(baseDir: string): Plugin {
  let isGenerating = false

  async function generateBridgeClient() {
    if (isGenerating) return
    isGenerating = true

    const outputDir = resolve(baseDir, '.soma/bridge-client')

    // Ensure output directory exists
    mkdirSync(outputDir, { recursive: true })

    console.log('Generating Bridge client from OpenAPI spec...')

    return new Promise<void>((resolvePromise, reject) => {
      const child = spawn(
        'npx',
        [
          '--yes',
          '@openapitools/openapi-generator-cli',
          'generate',
          '-i',
          'http://localhost:3000/api/bridge/v1/function-instances/openapi.json',
          '-t',
          './node_modules/@soma/sdk/openapi-template',
          '-g',
          'typescript-fetch',
          '--additional-properties',
          'supportsES6=true',
          '--ignore-file-override=./node_modules/@soma/sdk/openapi-template/.openapi-generator-ignore',
          '-o',
          outputDir,
        ],
        {
          stdio: 'inherit',
          cwd: baseDir,
          shell: true,
        }
      )

      child.on('error', (err) => {
        console.error('Failed to generate Bridge client:', err)
        isGenerating = false
        reject(err)
      })

      child.on('exit', (code) => {
        isGenerating = false
        if (code === 0) {
          console.log('Bridge client generated successfully!')
          resolvePromise()
        } else {
          const error = new Error(`openapi-generator-cli exited with code ${code}`)
          console.error(error.message)
          reject(error)
        }
      })
    })
  }

  return {
    name: 'soma-bridge-client',

    async configureServer(server) {
      // Generate bridge client before server starts
      server.httpServer?.once('listening', async () => {
        // Wait a bit for the bridge server to be ready
        await new Promise(resolve => setTimeout(resolve, 2000))
        try {
          await generateBridgeClient()
        } catch (err) {
          console.error('Bridge client generation failed:', err)
          // Don't fail the dev server, just log the error
        }
      })
    },
  }
}

/**
 * Plugin to generate and run standalone server in dev mode
 * Note: Runs in a separate Node process to avoid conflicts with Vite's module system
 */
function standaloneServerPlugin(baseDir: string): Plugin {
  let serverProcess: ChildProcess | null = null
  const standaloneFilePath = resolve(baseDir, '.soma/standalone.ts')

  function regenerateStandalone() {
    const content = generateStandaloneServer(baseDir, true)
    writeFileSync(standaloneFilePath, content)
    console.log('Generated standalone.ts')
  }

  function restartServer() {
    if (serverProcess) {
      console.log('Restarting SDK server...')
      serverProcess.kill()
      serverProcess = null
    }

    // Start the server in a separate Node process
    // We use tsx to transpile TypeScript on-the-fly since the standalone.ts
    // imports source .ts files directly (functions/agents)
    serverProcess = spawn('npx', ['tsx', standaloneFilePath], {
      stdio: 'inherit',
      cwd: baseDir,
      shell: true,
    })

    serverProcess.on('error', (err) => {
      console.error('Failed to start SDK server:', err)
    })

    serverProcess.on('exit', (code) => {
      if (code !== null && code !== 0) {
        console.error(`SDK server exited with code ${code}`)
      }
    })
  }

  return {
    name: 'soma-standalone-server',

    configureServer(devServer) {
      // Generate standalone file on startup
      regenerateStandalone()

      // Watch for changes in functions and agents directories
      const functionsDir = resolve(baseDir, 'functions')
      const agentsDir = resolve(baseDir, 'agents')

      devServer.watcher.add([functionsDir, agentsDir])

      devServer.watcher.on('add', (file) => {
        if (file.includes('/functions/') || file.includes('/agents/')) {
          console.log(`New file detected: ${file}`)
          regenerateStandalone()
          if (serverProcess) {
            restartServer()
          }
        }
      })

      devServer.watcher.on('unlink', (file) => {
        if (file.includes('/functions/') || file.includes('/agents/')) {
          console.log(`File removed: ${file}`)
          regenerateStandalone()
          if (serverProcess) {
            restartServer()
          }
        }
      })

      devServer.watcher.on('change', (file) => {
        if (file.includes('/functions/') || file.includes('/agents/')) {
          console.log(`File changed: ${file}`)
          regenerateStandalone()
          if (serverProcess) {
            restartServer()
          }
        }
      })

      // Start server after Vite is ready
      devServer.httpServer?.once('listening', () => {
        setTimeout(() => {
          console.log('\nStarting SDK server...')
          restartServer()
        }, 1000)
      })
    },

    buildEnd() {
      // Kill server process when build ends
      if (serverProcess) {
        serverProcess.kill()
        serverProcess = null
      }
    },
  }
}

/**
 * Vite plugin to generate manifest.json with full controller information
 * Uses a separate loader script to safely import modules
 */
function generateManifestPlugin(baseDir: string): Plugin {
  return {
    name: 'soma-manifest-generator',

    async writeBundle(options, bundle) {
      const outDir = options.dir || '.soma/build/js'
      const manifest: Manifest = {
        function_controllers: [],
        provider_controllers: [],
        agents: [],
      }

      // Track provider controllers to deduplicate
      const seenProviderControllers = new Map<string, ProviderController>()

      // Collect all function and agent files
      const functionFiles: string[] = []
      const agentFiles: string[] = []

      for (const [fileName, chunk] of Object.entries(bundle)) {
        if (chunk.type !== 'chunk') continue

        if (fileName.startsWith('functions/')) {
          functionFiles.push(fileName)
        } else if (fileName.startsWith('agents/')) {
          agentFiles.push(fileName)
        }
      }

      // Load each function module to extract metadata
      for (const fileName of functionFiles) {
        try {
          // Use dynamic import with a fresh module cache to avoid side effects
          const modulePath = resolve(outDir, fileName)
          // Use file:// protocol and add cache busting
          const moduleUrl = `file://${modulePath}?t=${Date.now()}`

          const module = await import(moduleUrl)
          // The default export might be a Promise (if using top-level await in source)
          let defaultExport = module.default
          if (defaultExport instanceof Promise) {
            defaultExport = await defaultExport
          }

          if (!defaultExport) {
            console.warn(`No default export found in ${fileName}`)
            continue
          }

          // Extract function controller
          if (defaultExport.functionController) {
            manifest.function_controllers.push({
              ...defaultExport.functionController,
              file: fileName,
            })
          } else {
            console.warn(`No functionController found in ${fileName}`)
          }

          // Extract and deduplicate provider controller
          if (defaultExport.providerController) {
            const providerController = defaultExport.providerController
            let foundDuplicate = false

            for (const [existingKey, existingProvider] of seenProviderControllers) {
              if (areProviderControllersEqual(providerController, existingProvider)) {
                foundDuplicate = true
                break
              }
              else if (existingKey === providerController.typeId) {
                throw new Error(`Duplicate provider controller typeId: ${providerController.typeId} in ${fileName}`)
              }
            }

            if (!foundDuplicate) {
              // Remove non-serializable properties
              const serializableProvider = JSON.parse(JSON.stringify(providerController, (key, value) => {
                if (key === 'invoke' || typeof value === 'function') return undefined
                return value
              }))

              const key = serializableProvider.type_id || serializableProvider.typeId || serializableProvider.name
              seenProviderControllers.set(key, serializableProvider)
            }
          } else {
            console.warn(`No providerController found in ${fileName}`)
          }
        } catch (err) {
          const errorMessage = getErrorMessage(err)
          console.warn(`Could not load ${fileName}:`, errorMessage)
          // Still add a placeholder entry so we know the file exists
          manifest.function_controllers.push({
            file: fileName,
            error: errorMessage,
          })
        }
      }

      // Load each agent module to extract metadata
      for (const fileName of agentFiles) {
        try {
          // Use dynamic import with a fresh module cache to avoid side effects
          const modulePath = resolve(outDir, fileName)
          // Use file:// protocol and add cache busting
          const moduleUrl = `file://${modulePath}?t=${Date.now()}`

          const module = await import(moduleUrl)
          // The default export might be a Promise (if using top-level await in source)
          let defaultExport = module.default
          if (defaultExport instanceof Promise) {
            defaultExport = await defaultExport
          }

          if (!defaultExport) {
            console.warn(`No default export found in ${fileName}`)
            continue
          }

          // Extract agent metadata (all fields from createSomaAgent)
          const agentMetadata: AgentMetadata = {
            file: fileName,
          }

          // Copy all fields from the agent object
          for (const [key, value] of Object.entries(defaultExport)) {
            // Skip non-serializable properties
            if (typeof value === 'function') continue

            // For handlers, just include the handler names
            if (key === 'handlers' && typeof value === 'object' && value !== null) {
              agentMetadata.handlers = Object.keys(value)
            } else {
              agentMetadata[key] = value
            }
          }

          manifest.agents.push(agentMetadata)
        } catch (err) {
          const errorMessage = getErrorMessage(err)
          console.warn(`Could not load ${fileName}:`, errorMessage)
          // Still add a placeholder entry so we know the file exists
          manifest.agents.push({
            file: fileName,
            error: errorMessage,
          })
        }
      }

      // Convert provider controllers map to array
      manifest.provider_controllers = Array.from(seenProviderControllers.values())

      // Write manifest.json
      const manifestPath = resolve(outDir, 'manifest.json')
      writeFileSync(manifestPath, JSON.stringify(manifest, null, 2))

      // Generate standalone.js for production
      const standaloneContent = generateStandaloneServer(baseDir, false)
      const standalonePath = resolve(outDir, 'standalone.js')
      writeFileSync(standalonePath, standaloneContent)

      console.log(`\nGenerated manifest.json with:`)
      console.log(`  - ${manifest.function_controllers.length} function controllers`)
      console.log(`  - ${manifest.provider_controllers.length} provider controllers`)
      console.log(`  - ${manifest.agents.length} agents`)
      console.log(`Generated standalone.js`)
    },
  }
}

/**
 * Create Soma Vite config
 * @param baseDir - The base directory of the project (usually __dirname from vite.config.ts)
 */
export function createSomaViteConfig(baseDir: string) {
  return defineConfig(({ command }) => {
    const isDev = command === 'serve'

    return {
      plugins: [
        isDev && bridgeClientPlugin(baseDir),
        isDev && standaloneServerPlugin(baseDir),
        generateManifestPlugin(baseDir),
      ].filter(Boolean),
      build: {
        target: 'node18',
        outDir: '.soma/build/js',
        emptyOutDir: true,
        minify: false,
        ssr: true, // Enable SSR mode for Node.js builds
        lib: {
          entry: buildSomaEntries(baseDir),
          formats: ['es'],
        },
        rollupOptions: {
          external: [
            // Don't bundle Node.js built-ins
            /^node:/,
            'fs',
            'path',
            'url',
            'stream',
            'util',
            'events',
            'buffer',
            'crypto',
            'http',
            'https',
            'zlib',
            'net',
            'tls',
            'os',
            'process',
            'child_process',
            'assert',
            'dns',
            'dgram',
            'readline',
            'repl',
            'tty',
            'v8',
            'vm',
            'worker_threads',
          ],
          output: {
            entryFileNames: '[name].js',
            chunkFileNames: 'chunks/[name]-[hash].js',
            // Preserve directory structure
            preserveModules: false,
            manualChunks: undefined,
          },
          treeshake: {
            preset: 'recommended',
            moduleSideEffects: false,
          },
        },
      },
      resolve: {
        conditions: ['node', 'import'],
      },
    }
  })
}
