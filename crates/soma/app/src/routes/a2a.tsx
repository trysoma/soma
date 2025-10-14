import { createFileRoute, Outlet } from '@tanstack/react-router'
import { A2aProvider } from '@/context/a2a'
import { SubNavigation } from '@/components/layout/sub-navigation'
import { LINKS } from '@/lib/links'

export const Route = createFileRoute('/a2a')({
  component: LayoutComponent,
})

function LayoutComponent() {
  return (
    <A2aProvider>
      <SubNavigation items={[
        {
          label: 'Overview',
          href: LINKS.A2A(),
        },
        {
          label: 'Chat',
          href: LINKS.A2A_CHAT(),
        }
      ]}
      nestLevel='second' />
      <Outlet />
    </A2aProvider>
  )
}
