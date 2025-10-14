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
import { useState, useRef } from 'react';
import { useA2a } from '@/context/a2a';
import { Tool, ToolContent, ToolHeader, ToolInput, ToolOutput } from './ai-elements/tool';

export const Example = () => {
  const [text, setText] = useState<string>('');
  const { sendMessage, currentTask, selectedIds } = useA2a();
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const messages = currentTask?.aiSdkView.messages ?? [];

  const handleSubmit = (message: PromptInputMessage) => {
    // TODO: handle attachments

    if (message.text) {
      sendMessage(message.text);
      setText('');
    }
  };

  console.log('messages', messages);

  // Show message if no context is selected
  if (!selectedIds.contextId) {
    return (
      <div className="relative flex size-full flex-col items-center justify-center overflow-hidden">
        <div className="text-center">
          <p className="text-lg text-muted-foreground">Create a context to get started</p>
        </div>
      </div>
    );
  }

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

export default Example;