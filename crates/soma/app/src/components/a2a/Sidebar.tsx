import type { Task, TaskState } from "@a2a-js/sdk";
import { ChevronLeft, MessageSquare } from "lucide-react";
import React from "react";

import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Sheet, SheetContent } from "@/components/ui/sheet";
import type { ChatContext } from "@/types/a2a";

interface SidebarProps {
  open: boolean;
  chatContexts: { [contextId: string]: ChatContext };
  selectedContextId: string | undefined;
  selectedTaskId: string | undefined;
  selectedArtifactId: string | undefined;
  onContextSelect: (contextId: string) => void;
  onTaskSelect: (taskId: string) => void;
  onArtifactSelect: (artifactId: string) => void;
  onNewChat: () => void;
  onClose: () => void;
}

const getTaskStateText = (state: TaskState): string => {
  switch (state) {
    case "submitted":
      return "Submitted";
    case "working":
      return "Working";
    case "input-required":
      return "Input Required";
    case "completed":
      return "Completed";
    case "canceled":
      return "Canceled";
    case "failed":
      return "Failed";
    case "rejected":
      return "Rejected";
    case "auth-required":
      return "Auth Required";
    case "unknown":
      return "Unknown";
    default:
      return "Unknown";
  }
};

export const Sidebar: React.FC<SidebarProps> = ({
  open,
  chatContexts,
  selectedContextId,
  selectedTaskId,
  selectedArtifactId,
  onContextSelect,
  onTaskSelect,
  onArtifactSelect,
  onNewChat,
  onClose,
}) => {
  const selectedContext: ChatContext | undefined = selectedContextId
    ? chatContexts[selectedContextId]
    : undefined;

  const selectedTask: Task | undefined =
    selectedContext && selectedTaskId
      ? selectedContext.tasks.find((task) => task.id === selectedTaskId)
      : undefined;

  return (
    <Sheet open={open} onOpenChange={onClose}>
      <SheetContent side="left" className="w-72 p-0">
        <div className="flex h-full flex-col">
          <div className="flex items-center justify-end border-b p-2">
            <Button variant="ghost" size="icon" onClick={onClose}>
              <ChevronLeft className="h-5 w-5" />
            </Button>
          </div>

          <div className="p-4">
            <Button
              onClick={onNewChat}
              variant="outline"
              className="w-full justify-start"
            >
              <MessageSquare className="mr-2 h-4 w-4" />
              New chat
            </Button>
          </div>

          <ScrollArea className="flex-1">
            {chatContexts && Object.keys(chatContexts).length > 0 && (
              <>
                <div className="px-4 pb-2">
                  <h2 className="text-sm font-semibold text-muted-foreground">Contexts</h2>
                </div>
                <div className="space-y-1 px-2">
                  {Object.values(chatContexts)
                    .reverse()
                    .map((context: ChatContext) => (
                      <Button
                        key={context.contextId}
                        variant={selectedContextId === context.contextId ? "secondary" : "ghost"}
                        className="w-full justify-start"
                        onClick={() => onContextSelect(context.contextId)}
                      >
                        <div className="flex flex-col items-start overflow-hidden">
                          <span className="truncate text-sm">{context.contextId}</span>
                          <span className="text-xs text-muted-foreground">{context.agent.name}</span>
                        </div>
                      </Button>
                    ))}
                </div>

                {selectedContext && selectedContext.tasks && selectedContext.tasks.length > 0 && (
                  <>
                    <Separator className="my-4" />
                    <div className="px-4 pb-2">
                      <h2 className="text-sm font-semibold text-muted-foreground">Tasks</h2>
                    </div>
                    <div className="space-y-1 px-2">
                      {selectedContext.tasks.map((task) => (
                        <Button
                          key={task.id}
                          variant={selectedTaskId === task.id ? "secondary" : "ghost"}
                          className="w-full justify-start"
                          onClick={() => onTaskSelect(task.id)}
                        >
                          <div className="flex flex-col items-start overflow-hidden">
                            <span className="truncate text-sm">{task.id}</span>
                            <span className="text-xs text-muted-foreground">
                              {getTaskStateText(task.status.state)}
                            </span>
                          </div>
                        </Button>
                      ))}
                    </div>
                  </>
                )}

                {selectedTask && selectedTask.artifacts && selectedTask.artifacts.length > 0 && (
                  <>
                    <Separator className="my-4" />
                    <div className="px-4 pb-2">
                      <h2 className="text-sm font-semibold text-muted-foreground">Artifacts</h2>
                    </div>
                    <div className="space-y-1 px-2">
                      {selectedTask.artifacts.map((artifact) => (
                        <Button
                          key={artifact.artifactId}
                          variant={selectedArtifactId === artifact.artifactId ? "secondary" : "ghost"}
                          className="w-full justify-start"
                          onClick={() => onArtifactSelect(artifact.artifactId)}
                        >
                          <div className="flex flex-col items-start overflow-hidden">
                            <span className="truncate text-sm">{artifact.artifactId}</span>
                            <span className="text-xs text-muted-foreground">{artifact.name}</span>
                          </div>
                        </Button>
                      ))}
                    </div>
                  </>
                )}
              </>
            )}
          </ScrollArea>
        </div>
      </SheetContent>
    </Sheet>
  );
};
