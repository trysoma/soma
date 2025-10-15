'use client';

import {
  Branch,
  BranchMessages,
  BranchNext,
  BranchPage,
  BranchPrevious,
  BranchSelector,
} from '@/components/ai-elements/branch';
import {
  Conversation,
  ConversationContent,
  ConversationScrollButton,
} from '@/components/ai-elements/conversation';
import {
  PromptInput,
  PromptInputAttachment,
  PromptInputAttachments,
  PromptInputBody,
  type PromptInputMessage,
  PromptInputSubmit,
  PromptInputTextarea,
  PromptInputToolbar,
  PromptInputTools,
} from '@/components/ai-elements/prompt-input';
import {
  Message,
  MessageAvatar,
  MessageContent,
} from '@/components/ai-elements/message';
import {
  Reasoning,
  ReasoningContent,
  ReasoningTrigger,
} from '@/components/ai-elements/reasoning';
import { Response } from '@/components/ai-elements/response';
import {
  Source,
  Sources,
  SourcesContent,
  SourcesTrigger,
} from '@/components/ai-elements/sources';
import { useState, useRef, type ReactNode } from 'react';
import { useA2a } from '@/context/a2a';
import { Tool, ToolContent, ToolHeader, ToolInput, ToolOutput } from './ai-elements/tool';
import { Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenuButton, SidebarMenuItem, SidebarMenu, SidebarProvider, SidebarTrigger, useSidebar, SidebarRail } from '@/components/ui/sidebar';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { ArrowLeftFromLine, ArrowRightFromLine, Plus } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { v4 } from 'uuid';
import { cn } from '@/lib/utils';

export const A2aChat = () => {
  const [text, setText] = useState<string>('');
  const { sendMessage, currentTask } = useA2a();
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const messages = currentTask?.aiSdkView.messages ?? [];

  const handleSubmit = async (message: PromptInputMessage) => {
    // TODO: handle attachments

    if (message.text) {
      try {
        setText('');
        sendMessage(message.text);
      } catch (error) {
        console.error('Failed to send message:', error);
      }
    }
  };


  // Show message if no context is selected
  // if (!selectedIds.contextId) {
  //   return (
  //     <div className="relative flex size-full flex-col items-center justify-center overflow-hidden">
  //       <div className="text-center">
  //         <p className="text-lg text-muted-foreground">Create a context to get started</p>
  //       </div>
  //     </div>
  //   );
  // }

  return (
    <div className="relative flex size-full flex-col  overflow-hidden">
      <Conversation>
        <ConversationContent>
          {messages.map(({ versions, ...message }) => {
            // If no versions, create a single empty version for rendering
            const versionsToRender = versions.length > 0 ? versions : [{ id: '0', content: '' }];
            
            return (
              <Branch defaultBranch={0} key={message.key}>
                <BranchMessages>
                  {versionsToRender.map((version) => (
                    <Message
                      from={message.from}
                      key={`${message.key}-${version.id}`}
                    >
                      <div>
                        {message.sources?.length && (
                          <Sources>
                            <SourcesTrigger count={message.sources.length} />
                            <SourcesContent>
                              {message.sources.map((source) => (
                                <Source
                                  href={source.href}
                                  key={source.href}
                                  title={source.title}
                                />
                              ))}
                            </SourcesContent>
                          </Sources >
                        )}
                        {message.reasoning && (
                          <Reasoning duration={message.reasoning.duration}>
                            <ReasoningTrigger />
                            <ReasoningContent>
                              {message.reasoning.content}
                            </ReasoningContent>
                          </Reasoning>
                        )}
                        {message.tools && message.tools.length > 0 && (
                          <div className="mb-2">
                            {message.tools.map((tool, index) => (
                              <Tool key={`${message.key}-tool-${index}`}>
                                <ToolHeader type={tool.type} state={tool.state} />
                                <ToolContent>
                                  <ToolInput input={tool.parameters} />
                                  {tool.state === 'output-available' && (
                                    <ToolOutput errorText={tool.error} output={tool.result} />
                                  )}
                                </ToolContent>
                              </Tool>
                            ))}
                          </div>
                        )}
                        {version.content && (
                          <MessageContent>
                            <Response>{version.content}</Response>
                          </MessageContent>
                        )}
                      </div>
                      <MessageAvatar name={message.name} src={message.avatar} />
                    </Message>
                  ))}
                </BranchMessages>
                {versions.length > 1 && (
                  <BranchSelector from={message.from}>
                    <BranchPrevious />
                    <BranchPage />
                    <BranchNext />
                  </BranchSelector>
                )}
              </Branch>
            );
          })}
        </ConversationContent>
        <ConversationScrollButton />
      </Conversation>
      <div className="grid shrink-0 gap-4 pt-4">
        <div className="w-full max-w-xl mx-auto px-4 pb-4">
          <PromptInput globalDrop multiple onSubmit={handleSubmit} className="bg-white rounded-md">
            <PromptInputBody>
              <PromptInputAttachments>
                {(attachment) => <PromptInputAttachment data={attachment} />}
              </PromptInputAttachments>
              <PromptInputTextarea
                onChange={(event) => setText(event.target.value)}
                ref={textareaRef}
                value={text}
              />
            </PromptInputBody>
            <PromptInputToolbar>
              <PromptInputTools>
                
              </PromptInputTools>
              <PromptInputSubmit disabled={!text.trim()} />
            </PromptInputToolbar>
          </PromptInput>
        </div>
      </div>
    </div>
  );
};

// Layout component with sidebars
export function A2aChatLayout() {
  const { contexts, createContext, currentContext, setCurrentContext, setCurrentTask, selectedIds } = useA2a();
  const tasks = currentContext?.tasks ?? [];
  
  return (
    <div className='flex h-auto w-full'>
      <ListSidebar
        list={contexts}
        setCurrent={setCurrentContext}
        createItem={createContext} 
        currentId={selectedIds.contextId}
        title="Contexts"
        mainButton={
          <div className='flex flex-col gap-2'>
            <Button variant="outline" className='w-full overflow-hidden' onClick={() => {
              createContext(v4());
            }}>
              <Plus className="size-4" />
              Create context
            </Button>
            <Button variant="outline" className='w-full overflow-hidden' onClick={() => {
              setCurrentContext(null);
            }}>
              Reset context
            </Button>
          </div>
        }
      />
      <SidebarProvider>
        <ListSidebar
          list={tasks.map((task) => task.somaView)}
          setCurrent={(taskId) => setCurrentTask(taskId)}
          createItem={() => { }} 
          currentId={selectedIds.taskId}
          title="Tasks"
          mainButton={
            <Button variant="outline" className='w-full overflow-hidden' onClick={() => {
              setCurrentTask(null);
            }}>
              Reset task ID
            </Button>
          }
        />
        <main className="h-full max-h-[calc(100vh-var(--header-height)-var(--nav-height)-var(--sub-nav-height))] overflow-y-scroll w-full flex-1">
          <A2aChat />
        </main>
      </SidebarProvider>
    </div>
  );
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

function ListSidebar<T extends BaseItem>({ list, setCurrent, currentId, mainButton, title }: ListSidebarProps<T>) {
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
          {state === "expanded" && (
            <SidebarMenuItem>
              <div className='grid flex-1'>
                {mainButton}
              </div>
            </SidebarMenuItem>
          )}
          {list
            .sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime())
            .map((item, index) => (
              <SidebarMenuItem key={item.id}>
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

export default A2aChat;