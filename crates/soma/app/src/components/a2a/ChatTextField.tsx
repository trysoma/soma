import { ArrowUp, Square } from "lucide-react";
import React from "react";

import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";

interface ChatTextFieldProps {
  value: string;
  loading?: boolean;
  autoFocus?: boolean;
  onChange: (value: string) => void;
  onSendMessage: (message: string) => void;
}

export const ChatTextField: React.FC<ChatTextFieldProps> = ({
  value,
  loading = false,
  autoFocus = false,
  onChange,
  onSendMessage,
}) => {
  const inputRef = React.useRef<HTMLTextAreaElement>(null);

  const handleSend = (): void => {
    if (value.trim() && !loading) {
      onSendMessage(value.trim());
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>): void => {
    onChange(e.target.value);
  };

  const handleKeyPress = (event: React.KeyboardEvent): void => {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      handleSend();
    }
  };

  // Focus the text field when autoFocus changes to true
  React.useEffect(() => {
    if (autoFocus && inputRef.current) {
      inputRef.current.focus();
    }
  }, [autoFocus]);

  return (
    <div className="relative">
      <Textarea
        ref={inputRef}
        value={value}
        onChange={handleChange}
        onKeyDown={handleKeyPress}
        placeholder="Ask anything"
        className="min-h-[60px] resize-none pr-12"
        rows={1}
      />
      <Button
        onClick={handleSend}
        disabled={!value.trim() && !loading}
        size="icon"
        className="absolute bottom-2 right-2 h-8 w-8 rounded-full"
      >
        {loading ? <Square className="h-4 w-4" /> : <ArrowUp className="h-4 w-4" />}
      </Button>
    </div>
  );
};
