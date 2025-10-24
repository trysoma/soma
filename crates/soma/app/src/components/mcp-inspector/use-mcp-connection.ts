import { useState, useCallback } from 'react'
import { Client } from '@modelcontextprotocol/sdk/client/index.js'
import { SSEClientTransport } from '@modelcontextprotocol/sdk/client/sse.js'
import type {
  ClientRequest,
  ServerCapabilities,
  ServerNotification
} from '@modelcontextprotocol/sdk/types.js'
import type { Transport } from '@modelcontextprotocol/sdk/shared/transport.js'
import { z } from 'zod'

type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error'

interface UseMCPConnectionOptions {
  serverUrl: string
}

interface RequestHistoryItem {
  request: string
  response?: string
}

export function useMCPConnection({ serverUrl }: UseMCPConnectionOptions) {
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>('disconnected')
  const [serverCapabilities, setServerCapabilities] = useState<ServerCapabilities | null>(null)
  const [mcpClient, setMcpClient] = useState<Client | null>(null)
  const [clientTransport, setClientTransport] = useState<Transport | null>(null)
  const [requestHistory, setRequestHistory] = useState<RequestHistoryItem[]>([])
  const [notifications, setNotifications] = useState<ServerNotification[]>([])

  const pushHistory = useCallback((request: object, response?: object) => {
    setRequestHistory((prev) => [
      ...prev,
      {
        request: JSON.stringify(request),
        response: response !== undefined ? JSON.stringify(response) : undefined
      }
    ])
  }, [])

  const connect = useCallback(async () => {
    try {
      setConnectionStatus('connecting')

      const clientCapabilities = {
        capabilities: {
          sampling: {},
          roots: {
            listChanged: true
          }
        }
      }

      const client = new Client(
        { name: 'mcp-inspector', version: '1.0.0' },
        clientCapabilities
      )

      const transport = new SSEClientTransport(new URL(serverUrl))

      await client.connect(transport as Transport)
      setClientTransport(transport)

      const capabilities = client.getServerCapabilities()
      setServerCapabilities(capabilities ?? null)

      pushHistory(
        { method: 'initialize' },
        {
          capabilities,
          serverInfo: client.getServerVersion()
        }
      )

      // Set up notification handler
      client.fallbackNotificationHandler = (notification: any): Promise<void> => {
        setNotifications((prev) => [...prev, notification as ServerNotification])
        return Promise.resolve()
      }

      setMcpClient(client)
      setConnectionStatus('connected')
    } catch (error) {
      console.error('Failed to connect to MCP server:', error)
      setConnectionStatus('error')
    }
  }, [serverUrl, pushHistory])

  const disconnect = useCallback(async () => {
    await mcpClient?.close()
    setMcpClient(null)
    setClientTransport(null)
    setConnectionStatus('disconnected')
    setServerCapabilities(null)
  }, [mcpClient, clientTransport])

  const makeRequest = useCallback(async (request: ClientRequest, schema?: any): Promise<any> => {
    if (!mcpClient) {
      throw new Error('MCP client not connected')
    }

    try {
      // If no schema provided, use a passthrough object schema
      const responseSchema = schema || z.object({}).catchall(z.any())
      const response = await mcpClient.request(request as any, responseSchema)
      pushHistory(request, response)
      return response
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      pushHistory(request, { error: errorMessage })
      throw error
    }
  }, [mcpClient, pushHistory])

  const clearRequestHistory = useCallback(() => {
    setRequestHistory([])
  }, [])

  const clearNotifications = useCallback(() => {
    setNotifications([])
  }, [])

  return {
    connectionStatus,
    serverCapabilities,
    requestHistory,
    notifications,
    connect,
    disconnect,
    makeRequest,
    clearRequestHistory,
    clearNotifications
  }
}
