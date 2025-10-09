import type { DataPart, FilePart, Message, TextPart } from "@a2a-js/sdk";
import React from "react";

import { TextDataPartMarkdown } from "@/components/a2a/TextDataPartMarkdown";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";

interface ToolCallAccordionProps {
  toolCallMessage: Message;
  toolCallResultMessage: Message | undefined;
}

export const ToolCallAccordion: React.FC<ToolCallAccordionProps> = ({
  toolCallMessage,
  toolCallResultMessage,
}) => {
  if (!toolCallMessage.metadata?.toolCallId || !toolCallMessage.metadata?.toolCallName) {
    console.error("`toolCallMessage` `metadata` is missing `toolCallId` or `toolCallName`");
    return null;
  }

  const argsDataPart: TextPart | DataPart | FilePart | undefined = toolCallMessage.parts[0];

  if (!argsDataPart || argsDataPart.kind !== "data") {
    console.error("`toolCallMessage` `parts[0]` should be a `DataPart`");
    return null;
  }

  const resultDataPart: TextPart | DataPart | FilePart | undefined =
    toolCallResultMessage?.parts[0];

  if (resultDataPart && resultDataPart.kind !== "text" && resultDataPart.kind !== "data") {
    console.error(
      "If `toolCallResultMessage` exists, `parts[0]` should be a `TextPart` or `DataPart`"
    );
    return null;
  }

  return (
    <Accordion type="single" collapsible className="border rounded-lg">
      <AccordionItem value="item-1" className="border-none">
        <AccordionTrigger className="px-4 hover:no-underline">
          <div>
            <h3 className="text-lg font-semibold">
              {toolCallMessage.metadata?.toolCallName as string}
            </h3>
            <p className="text-sm text-muted-foreground">
              {toolCallMessage.metadata?.toolCallId as string}
            </p>
          </div>
        </AccordionTrigger>
        <AccordionContent className="px-4 pb-4">
          <TextDataPartMarkdown part={argsDataPart} />

          {resultDataPart && (
            <div className="mt-2">
              <TextDataPartMarkdown part={resultDataPart} />
            </div>
          )}
        </AccordionContent>
      </AccordionItem>
    </Accordion>
  );
};
