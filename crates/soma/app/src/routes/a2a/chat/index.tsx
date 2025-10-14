import { Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenuButton, SidebarMenuItem, SidebarMenu, SidebarProvider, SidebarTrigger, useSidebar, SidebarRail } from '@/components/ui/sidebar'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { createFileRoute } from '@tanstack/react-router'
import { ArrowLeftFromLine, ArrowRightFromLine, Plus } from 'lucide-react';
import { Example } from '@/components/example';
import { useA2a } from '@/context/a2a';
import { v4 } from 'uuid';
import { Button } from '@/components/ui/button';
import type { ReactNode } from 'react';
import { cn } from '@/lib/utils';
export const Route = createFileRoute('/a2a/chat/')({
  component: RouteComponent,
})

function RouteComponent() {

  return <>
    <SidebarProvider >
      <RouterComponentInner />
    </SidebarProvider>
  </>
}

function RouterComponentInner() {
  const { contexts, createContext, currentContext, setCurrentContext, setCurrentTask, selectedIds } = useA2a();
  const tasks = currentContext?.tasks ?? [];
  return (
    <>
      <div className='flex h-auto  w-full'>
        <ListSidebar
          list={contexts}
          setCurrent={setCurrentContext}
          createItem={createContext} 
          currentId={selectedIds.contextId}
          title="Contexts"
          mainButton={<Button variant="outline" className='w-full overflow-hidden' onClick={() => {
            createContext(v4());
          }}
          >
            <Plus className="size-4" />
            Create Context
          </Button>}
        />
        <SidebarProvider>
          <ListSidebar
            list={tasks.map((task) => task.somaView)}
            setCurrent={(taskId) => setCurrentTask(taskId)}
            createItem={() => { }} 
            currentId={selectedIds.taskId}
            title="Tasks"
            mainButton={<Button variant="outline" className='w-full overflow-hidden' onClick={() => {
              setCurrentTask(null);
            }}>
              Reset task ID
            </Button>}
          />
          <main className="h-full max-h-[calc(100vh-var(--header-height)-var(--nav-height)-var(--sub-nav-height))] overflow-y-scroll w-full flex-1">
            <Example />
          </main>
        </SidebarProvider>
      </div>
    </>
  )
}

interface BaseItem {
  createdAt: Date;
  id: string;
}

interface ListSidebarProps<T extends BaseItem> {
  list: T[];
  setCurrent: (id: string | null) => void;
  createItem: (id: string) => void;
  currentId: string | null;
  mainButton: ReactNode;
  title: string;
}

export function ListSidebar<T extends BaseItem>({ list, setCurrent, currentId, mainButton, title }: ListSidebarProps<T>) {
  const { state } = useSidebar();

  return (
    <Sidebar className='sticky w-full' collapsible="icon">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <div
              className="flex data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground items-center justify-center"
            >
              {state === "expanded" ? (
                <>

                  <div className="grid flex-1 text-left text-sm leading-tight">
                    <span className="truncate font-semibold">{title}</span>
                  </div>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <SidebarTrigger>
                        <ArrowLeftFromLine className="size-4" />
                      </SidebarTrigger>
                    </TooltipTrigger>
                    <TooltipContent side="right">
                      Collapse sidebar
                    </TooltipContent>
                  </Tooltip>
                </>
              ) : (
                <>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <SidebarTrigger>
                        <ArrowRightFromLine className="size-4" />
                      </SidebarTrigger>
                    </TooltipTrigger>
                    <TooltipContent side="right">
                      Expand sidebar
                    </TooltipContent>
                  </Tooltip>
                </>
              )}
            </div>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <div className='grid flex-1'>
              {mainButton}
            </div>
          </SidebarMenuItem>
          {list
            .sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime())
            .map((item, index) => (
              <SidebarMenuItem key={item.createdAt.getTime()}>
                <SidebarMenuButton onClick={() => {
                  setCurrent(item.id);
                }}
                  className={cn('text-[0.6rem]', currentId === item.id && 'bg-sidebar-accent text-sidebar-accent-foreground')}>
                  {index + 1}. {state === "expanded" ? item.id : ""}
                </SidebarMenuButton>
              </SidebarMenuItem>
            ))}
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          {/* <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton className="mb-[-1rem]" onClick={() => {
                router.push("/");
              }}>
                <House className="size-4" />
                Dashboard
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu> */}
        </SidebarGroup>
        {/* <NavMain items={data.navMain} />
				<NavProjects projects={data.projects} /> */}
      </SidebarContent>
      <SidebarFooter>
        {/* <NavUser user={data.user} /> */}
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
