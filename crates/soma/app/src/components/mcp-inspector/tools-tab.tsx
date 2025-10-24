import { useState, useEffect } from 'react'
import { TabsContent } from '@/components/ui/tabs'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Send } from 'lucide-react'
import { ListPane } from './list-pane'
import { JsonView } from './json-view'
import type { Tool, CompatibilityCallToolResult } from '@modelcontextprotocol/sdk/types.js'

interface ToolsTabProps {
  tools: Tool[]
  listTools: () => void
  clearTools: () => void
  callTool: (name: string, params: Record<string, unknown>) => Promise<void>
  selectedTool: Tool | null
  setSelectedTool: (tool: Tool | null) => void
  toolResult: CompatibilityCallToolResult | null
}

export function ToolsTab({
  tools,
  listTools,
  clearTools,
  callTool,
  selectedTool,
  setSelectedTool,
  toolResult
}: ToolsTabProps) {
  const [params, setParams] = useState<Record<string, unknown>>({})
  const [isRunning, setIsRunning] = useState(false)

  useEffect(() => {
    if (selectedTool) {
      const defaultParams: Record<string, unknown> = {}
      const properties = selectedTool.inputSchema?.properties || {}
      Object.keys(properties).forEach((key) => {
        defaultParams[key] = ''
      })
      setParams(defaultParams)
    }
  }, [selectedTool])

  const handleRunTool = async () => {
    if (!selectedTool) return

    try {
      setIsRunning(true)
      await callTool(selectedTool.name, params)
    } finally {
      setIsRunning(false)
    }
  }

  return (
    <TabsContent value="tools" className="space-y-4">
      <ListPane
        items={tools}
        listItems={listTools}
        clearItems={clearTools}
        setSelectedItem={setSelectedTool}
        renderItem={(tool) => (
          <div className="flex flex-col items-start">
            <span className="font-medium">{tool.name}</span>
            <span className="text-sm text-muted-foreground line-clamp-2">
              {tool.description}
            </span>
          </div>
        )}
        title="Tools"
        buttonText="List Tools"
      />

      {selectedTool && (
        <div className="space-y-4">
          <div className="mb-4">
            <h3 className="font-semibold">{selectedTool.name}</h3>
            <p className="text-sm text-muted-foreground mt-2">
              {selectedTool.description}
            </p>
          </div>

          {Object.entries(selectedTool.inputSchema?.properties || {}).map(
            ([key, schema]: [string, any]) => (
              <div key={key}>
                <Label htmlFor={key}>{key}</Label>
                {schema.type === 'string' && schema.enum ? (
                  <select
                    id={key}
                    className="w-full px-3 py-2 border rounded-md"
                    value={String(params[key] || '')}
                    onChange={(e) =>
                      setParams({ ...params, [key]: e.target.value })
                    }
                  >
                    <option value="">Select...</option>
                    {schema.enum.map((option: string) => (
                      <option key={option} value={option}>
                        {option}
                      </option>
                    ))}
                  </select>
                ) : schema.type === 'string' ? (
                  <Textarea
                    id={key}
                    placeholder={schema.description}
                    value={String(params[key] || '')}
                    onChange={(e) =>
                      setParams({ ...params, [key]: e.target.value })
                    }
                  />
                ) : schema.type === 'number' || schema.type === 'integer' ? (
                  <Input
                    type="number"
                    id={key}
                    placeholder={schema.description}
                    value={String(params[key] || '')}
                    onChange={(e) =>
                      setParams({
                        ...params,
                        [key]: e.target.value ? Number(e.target.value) : undefined
                      })
                    }
                  />
                ) : schema.type === 'boolean' ? (
                  <div className="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      id={key}
                      checked={!!params[key]}
                      onChange={(e) =>
                        setParams({ ...params, [key]: e.target.checked })
                      }
                    />
                    <Label htmlFor={key}>{schema.description}</Label>
                  </div>
                ) : (
                  <Textarea
                    id={key}
                    placeholder="Enter JSON"
                    value={
                      typeof params[key] === 'object'
                        ? JSON.stringify(params[key], null, 2)
                        : String(params[key] || '')
                    }
                    onChange={(e) => {
                      try {
                        setParams({ ...params, [key]: JSON.parse(e.target.value) })
                      } catch {
                        setParams({ ...params, [key]: e.target.value })
                      }
                    }}
                  />
                )}
              </div>
            )
          )}

          <Button onClick={handleRunTool} disabled={isRunning}>
            {isRunning ? (
              <>Running...</>
            ) : (
              <>
                <Send className="w-4 h-4 mr-2" />
                Run Tool
              </>
            )}
          </Button>

          {toolResult && (
            <div>
              <h4 className="font-semibold mb-2">Result:</h4>
              <JsonView
                data={toolResult}
                isError={!!toolResult.isError}
              />
            </div>
          )}
        </div>
      )}
    </TabsContent>
  )
}
