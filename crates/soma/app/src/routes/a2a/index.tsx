import { createFileRoute } from '@tanstack/react-router'
import { useA2a } from '@/context/a2a';

export const Route = createFileRoute('/a2a/')({
  component: RouteComponent,
})

function RouteComponent() {
  const { agentCard } = useA2a();
  return <main>
    asd
  </main>
}
