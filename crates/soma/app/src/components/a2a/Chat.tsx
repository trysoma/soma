import type { Artifact, Message } from "@a2a-js/sdk";
import React from "react";

import { AIMessage } from "@/components/a2a/AIMessage";
import { ArtifactAccordion } from "@/components/a2a/ArtifactAccordion";
import { ChatTextField } from "@/components/a2a/ChatTextField";
import { Loading } from "@/components/a2a/Loading";
import { TaskDivider } from "@/components/a2a/TaskDivider";
import { ToolCallAccordion } from "@/components/a2a/ToolCallAccordion";
import { UserMessage } from "@/components/a2a/UserMessage";
import type { ChatContext } from "@/types/a2a";

interface TaskDividerItem {
  kind: "task-divider";
  taskId: string;
}

interface ToolCallItem {
  kind: "tool-call";
  toolCallMessage: Message;
  toolCallResultMessage: Message | undefined;
}

type ChatItem = Message | Artifact | TaskDividerItem | ToolCallItem;

interface ChatProps {
  activeChatContext?: ChatContext;
  scrollToTaskId?: string;
  scrollToArtifactId?: string;
  currentMessageText: string;
  autoFocusChatTextField?: boolean;
  onSendMessage: (message: string) => void;
  onChatTextFieldChange: (value: string) => void;
}

export const Chat: React.FC<ChatProps> = ({
  activeChatContext,
  scrollToTaskId,
  scrollToArtifactId,
  currentMessageText,
  autoFocusChatTextField = false,
  onSendMessage,
  onChatTextFieldChange,
}) => {
  const messagesEndRef = React.useRef<HTMLDivElement>(null);
  const taskRefs = React.useRef<Map<string, HTMLDivElement>>(new Map());
  const artifactRefs = React.useRef<Map<string, HTMLDivElement>>(new Map());

  // Get chat items (messages, artifacts, and task dividers) from the context and pending message
  const chatItems: ChatItem[] = React.useMemo(() => {
    const chatItems2: ChatItem[] = [];

    if (activeChatContext) {
      for (const task of activeChatContext.tasks) {
        // Add task divider at the start of each task
        chatItems2.push({
          kind: "task-divider",
          taskId: task.id,
        });

        // Combine history with status message
        let messages: Message[] = [];

        if (task.history) {
          messages = [...task.history];
        }

        if (task.status.message) {
          messages.push(task.status.message);
        }

        // Add messages to chat items
        for (const message of messages) {
          if (!message.metadata?.type) {
            chatItems2.push(message);
          } else if (message.metadata?.type === "tool-call") {
            const toolCallId: string = message.metadata.toolCallId as string;

            const toolCallResultMessage: Message | undefined = messages.find(
              (message) =>
                message.metadata?.type === "tool-call-result" &&
                message.metadata?.toolCallId === toolCallId
            );

            chatItems2.push({
              kind: "tool-call",
              toolCallMessage: message,
              toolCallResultMessage: toolCallResultMessage,
            });
          }
        }

        // Add artifacts if they exist
        if (task.artifacts) {
          chatItems2.push(...task.artifacts);
        }
      }

      // Add pending message for immediate display
      if (activeChatContext.pendingMessage) {
        chatItems2.push(activeChatContext.pendingMessage);
      }
    }

    return chatItems2;
  }, [activeChatContext]);

  const handleSendMessage = (message: string): void => {
    onSendMessage(message);
  };

  React.useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [chatItems]);

  React.useEffect(() => {
    if (scrollToTaskId) {
      const element = taskRefs.current.get(scrollToTaskId);

      if (element) {
        element.scrollIntoView({ behavior: "smooth", block: "start" });
      }
    }
  }, [scrollToTaskId]);

  React.useEffect(() => {
    if (scrollToArtifactId) {
      const element = artifactRefs.current.get(scrollToArtifactId);

      if (element) {
        element.scrollIntoView({ behavior: "smooth", block: "start" });
      }
    }
  }, [scrollToArtifactId]);

  return (
    <div className="flex h-full flex-col bg-background">
      {/* Messages */}
      <div className="flex-1 overflow-auto">
        <div className="container mx-auto max-w-3xl space-y-4 py-4">
          {chatItems.map((item: ChatItem) => {
            if ("kind" in item && item.kind === "task-divider") {
              const taskDividerItem: TaskDividerItem = item as TaskDividerItem;

              return (
                <div key={taskDividerItem.taskId} className="mb-4">
                  <TaskDivider
                    taskId={taskDividerItem.taskId}
                    onRef={(el) => {
                      if (el) {
                        taskRefs.current.set(taskDividerItem.taskId, el);
                      }
                    }}
                  />
                </div>
              );
            } else if ("kind" in item && item.kind === "message") {
              const message: Message = item as Message;

              return (
                <div key={message.messageId} className="mb-4">
                  {message.role === "user" ? (
                    <div className="flex justify-end">
                      <div className="max-w-[70%]">
                        <UserMessage message={message} />
                      </div>
                    </div>
                  ) : (
                    <div className="flex justify-start">
                      <AIMessage message={message} />
                    </div>
                  )}
                </div>
              );
            } else if ("kind" in item && item.kind === "tool-call") {
              const toolCallItem: ToolCallItem = item as ToolCallItem;

              return (
                <div key={toolCallItem.toolCallMessage.messageId} className="mb-4">
                  <ToolCallAccordion
                    toolCallMessage={toolCallItem.toolCallMessage}
                    toolCallResultMessage={toolCallItem.toolCallResultMessage}
                  />
                </div>
              );
            } else {
              const artifact: Artifact = item as Artifact;

              return (
                <div
                  key={artifact.artifactId}
                  className="mb-4"
                  ref={(el: HTMLDivElement | null) => {
                    if (el) {
                      artifactRefs.current.set(artifact.artifactId, el);
                    }
                  }}
                >
                  <ArtifactAccordion artifact={artifact} />
                </div>
              );
            }
          })}

          {activeChatContext?.loading && (
            <div className="mb-4">
              <Loading />
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>
      </div>

      {/* Chat Text Field */}
      <div className="border-t bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container mx-auto max-w-3xl py-4">
          <ChatTextField
            value={currentMessageText}
            loading={activeChatContext?.loading}
            autoFocus={autoFocusChatTextField}
            onChange={onChatTextFieldChange}
            onSendMessage={handleSendMessage}
          />
        </div>
      </div>
    </div>
  );
};
