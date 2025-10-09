import { createFileRoute } from '@tanstack/react-router'
import { ChatPage } from '@/components/a2a/ChatPage'

export const Route = createFileRoute('/chat/')({
  component: ChatPage,
})
