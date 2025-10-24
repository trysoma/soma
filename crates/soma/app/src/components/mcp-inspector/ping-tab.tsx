import { TabsContent } from '@/components/ui/tabs'
import { Button } from '@/components/ui/button'
import { Bell } from 'lucide-react'

interface PingTabProps {
  onPingClick: () => void
}

export function PingTab({ onPingClick }: PingTabProps) {
  return (
    <TabsContent value="ping">
      <div className="flex flex-col items-center justify-center p-8 space-y-4">
        <Bell className="w-16 h-16 text-muted-foreground" />
        <h3 className="text-lg font-semibold">Test Server Connection</h3>
        <p className="text-sm text-muted-foreground text-center max-w-md">
          Send a ping request to verify the MCP server is responding
        </p>
        <Button onClick={onPingClick}>
          <Bell className="w-4 h-4 mr-2" />
          Send Ping
        </Button>
      </div>
    </TabsContent>
  )
}
