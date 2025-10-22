import { createFileRoute } from '@tanstack/react-router'
import { SidebarProvider } from '@/components/ui/sidebar'
import { A2aChatLayout } from '@/components/a2a-chat'

export const Route = createFileRoute('/chat/')({
  component: RouteComponent,
})

function RouteComponent() {
  return (
    <SidebarProvider>
      <A2aChatLayout />
    </SidebarProvider>
  )
}
