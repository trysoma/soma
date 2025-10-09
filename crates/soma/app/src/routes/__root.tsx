import { Outlet, createRootRouteWithContext } from '@tanstack/react-router'
import { TanStackRouterDevtoolsPanel } from '@tanstack/react-router-devtools'
import { TanstackDevtools } from '@tanstack/react-devtools'
import ReactQueryProvider from '@/context/request-query-provider'
import { Header } from '@/components/layout/header'
import { Navigation } from '@/components/layout/navigation'

export interface RouterContext {
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: () => (
    <>
      <ReactQueryProvider>
        <div className="min-h-screen bg-background antialiased w-full mx-auto scroll-smooth font-sans">
            <Header />
            <Navigation />
            <Outlet />
            <TanstackDevtools
              config={{
                position: 'bottom-left',
              }}
              plugins={[
                {
                  name: 'Tanstack Roaaauter',
                  render: <TanStackRouterDevtoolsPanel />,
                },
              ]}
            />
          </div>
      </ReactQueryProvider>
    </>
  ),
})


