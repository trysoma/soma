import React from "react";

import { Chat } from "@/components/a2a/Chat";
import { InsetSidebar } from "@/components/a2a/InsetSidebar";
import { useChat } from "@/hooks/a2a/useChat";

export const ChatPage: React.FC = () => {
  const chat = useChat();

  if (chat.isLoadingAgent) {
    return (
      <div className="flex h-screen items-center justify-center">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent mx-auto mb-4" />
          <p className="text-muted-foreground">Loading agent...</p>
        </div>
      </div>
    );
  }

  if (chat.agentError) {
    return (
      <div className="flex h-screen items-center justify-center">
        <div className="text-center max-w-md">
          <p className="text-destructive mb-2">Error loading agent</p>
          <p className="text-sm text-muted-foreground">{chat.agentError.message}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen">
      <InsetSidebar
        chatContexts={chat.chatContexts.chatContexts}
        selectedContextId={chat.selected.selectedContextId}
        selectedTaskId={chat.selected.selectedTaskId}
        selectedArtifactId={chat.selected.selectedArtifactId}
        onContextSelect={chat.handleContextSelect}
        onTaskSelect={chat.handleTaskSelect}
        onArtifactSelect={chat.handleArtifactSelect}
        onNewChat={chat.handleNewChat}
      />

      <div className="flex-1 overflow-hidden">
        <Chat
          activeChatContext={chat.activeChatContext}
          scrollToTaskId={chat.scrolling.scrollToTaskId}
          scrollToArtifactId={chat.scrolling.scrollToArtifactId}
          currentMessageText={chat.currentMessageText}
          autoFocusChatTextField={chat.autoFocusChatTextField}
          onSendMessage={chat.handleSendMessage}
          onChatTextFieldChange={chat.handleMessageTextChange}
        />
      </div>
    </div>
  );
};
