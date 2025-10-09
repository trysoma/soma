import React from "react";

import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";

interface TaskDividerProps {
  taskId: string;
  onClick?: (taskId: string) => void;
  onRef?: (element: HTMLDivElement | null) => void;
}

export const TaskDivider: React.FC<TaskDividerProps> = ({ taskId, onClick, onRef }) => {
  const handleClick = (): void => {
    if (onClick) {
      onClick(taskId);
    }
  };

  return (
    <div ref={onRef} className="flex items-center gap-4">
      <Separator className="flex-1" />
      <Button variant="outline" size="sm" onClick={handleClick}>
        Task {taskId}
      </Button>
      <Separator className="flex-1" />
    </div>
  );
};
