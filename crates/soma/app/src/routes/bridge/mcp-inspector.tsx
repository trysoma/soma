import { createFileRoute } from '@tanstack/react-router'
import { MCPInspector } from '@/components/mcp-inspector/mcp-inspector'

export const Route = createFileRoute('/bridge/mcp-inspector')({
  component: MCPInspectorRoute,
})

function MCPInspectorRoute() {
  return <MCPInspector />
}
