import { useState, useEffect } from 'react'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Button } from '@/components/ui/button'
import { Hammer, Bell } from 'lucide-react'
import { ToolsTab } from './tools-tab'
import { PingTab } from './ping-tab'
import { HistoryAndNotifications } from './history-and-notifications'
import { useMCPConnection } from './use-mcp-connection'
import type {
  Tool,
  CompatibilityCallToolResult
} from '@modelcontextprotocol/sdk/types.js'

export function MCPInspector() {
  const [tools, setTools] = useState<Tool[]>([])
  const [selectedTool, setSelectedTool] = useState<Tool | null>(null)
  const [toolResult, setToolResult] = useState<CompatibilityCallToolResult | null>(null)
  const [activeTab, setActiveTab] = useState('tools')

  const {
    connectionStatus,
    serverCapabilities,
    requestHistory,
    notifications,
    connect,
    disconnect,
    makeRequest,
    clearRequestHistory,
    clearNotifications
  } = useMCPConnection({
    serverUrl: 'http://localhost:3000/api/bridge/v1/mcp'
  })

  useEffect(() => {
    // Auto-connect on mount
    if (connectionStatus === 'disconnected') {
      connect()
    }
  }, [])

  const listTools = async () => {
    try {
      const response = await makeRequest({
        method: 'tools/list' as const,
        params: {}
      })
      setTools(response.tools ?? [])
    } catch (error) {
      console.error('Failed to list tools:', error)
    }
  }

  const clearTools = () => {
    setTools([])
    setSelectedTool(null)
    setToolResult(null)
  }

  const callTool = async (name: string, params: Record<string, unknown>) => {
    try {
      const response = await makeRequest({
        method: 'tools/call' as const,
        params: {
          name,
          arguments: params
        }
      })
      setToolResult(response)
    } catch (e) {
      const errorResult: CompatibilityCallToolResult = {
        content: [{
          type: 'text',
          text: (e as Error).message ?? String(e)
        }],
        isError: true
      }
      setToolResult(errorResult)
    }
  }

  const sendPing = async () => {
    try {
      await makeRequest({
        method: 'ping' as const
      })
    } catch (error) {
      console.error('Failed to ping:', error)
    }
  }

  const isConnected = connectionStatus === 'connected'

  return (
    <div className="flex flex-col h-[calc(100vh-var(--header-height)-46px-var(--sub-nav-height))] ">
      <div className="flex items-center justify-between p-4 border-b flex-shrink-0">
        <div>
          <h1 className="text-2xl font-bold">MCP Inspector</h1>

        </div>
        <div className="flex gap-2">
          {connectionStatus === 'disconnected' && (
            <Button onClick={connect}>Connect</Button>
          )}
          {connectionStatus === 'connecting' && (
            <Button disabled>Connecting...</Button>
          )}
          {connectionStatus === 'connected' && (
            <Button onClick={disconnect} variant="outline">Disconnect</Button>
          )}
          {connectionStatus === 'error' && (
            <Button onClick={connect} variant="destructive">Retry Connection</Button>
          )}
        </div>
      </div>

      <div className="flex-1 min-h-0 grid grid-cols-2">
        <div className="overflow-y-auto">
          {isConnected ? (
            <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full p-4">
              <TabsList className="mb-4">
                <TabsTrigger
                  value="tools"
                  disabled={!serverCapabilities?.tools}
                >
                  <Hammer className="w-4 h-4 mr-2" />
                  Tools
                </TabsTrigger>
                <TabsTrigger value="ping">
                  <Bell className="w-4 h-4 mr-2" />
                  Ping
                </TabsTrigger>
              </TabsList>

              <ToolsTab
                tools={tools}
                listTools={listTools}
                clearTools={clearTools}
                callTool={callTool}
                selectedTool={selectedTool}
                setSelectedTool={setSelectedTool}
                toolResult={toolResult}
              />

              <PingTab onPingClick={sendPing} />
            </Tabs>
          ) : (
            <div className="flex items-center justify-center h-full">
              <p className="text-lg text-muted-foreground">
                {connectionStatus === 'disconnected' && 'Click Connect to start inspecting'}
                {connectionStatus === 'connecting' && 'Connecting to MCP server...'}
                {connectionStatus === 'error' && 'Failed to connect. Check if the server is running.'}
              </p>
            </div>
          )}
        </div>

        <div className="overflow-y-auto border-l">
          <HistoryAndNotifications
            requestHistory={requestHistory}
            serverNotifications={notifications}
            onClearHistory={clearRequestHistory}
            onClearNotifications={clearNotifications}
          />
        </div>
      </div>
    </div>
  )
}
