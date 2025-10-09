import type { DataPart, Message, TextPart } from "@a2a-js/sdk";

import { TextDataPartMarkdown } from "@/components/a2a/TextDataPartMarkdown";
import { Card } from "@/components/ui/card";

interface UserMessageProps {
  message: Message;
}

export const UserMessage: React.FC<UserMessageProps> = ({ message }) => {
  const textDataParts: (TextPart | DataPart)[] = message.parts.filter(
    (part) => part.kind === "text" || part.kind === "data"
  );

  return (
    <Card className="bg-muted p-4">
      {textDataParts.map((part, index) => (
        <TextDataPartMarkdown key={index} part={part} />
      ))}
    </Card>
  );
};
