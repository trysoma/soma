import type { DataPart, Message, TextPart } from "@a2a-js/sdk";

import { TextDataPartMarkdown } from "@/components/a2a/TextDataPartMarkdown";

interface AIMessageProps {
  message: Message;
}

export const AIMessage: React.FC<AIMessageProps> = ({ message }) => {
  const textDataParts: (TextPart | DataPart)[] = message.parts.filter(
    (part) => part.kind === "text" || part.kind === "data"
  );

  return (
    <>
      {textDataParts.map((part, index) => (
        <TextDataPartMarkdown key={index} part={part} />
      ))}
    </>
  );
};
