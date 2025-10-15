"use client";

import type { ReactNode } from "react";
import { Card } from "@/components/ui/card";
import { cn } from "@/lib/utils";

interface CardGridProps {
  children: ReactNode;
  className?: string;
  loading?: boolean;
  loadingCount?: number;
}

export function CardGrid({
  children,
  className,
  loading = false,
  loadingCount = 4
}: CardGridProps) {
  if (loading) {
    return (
      <div className={cn(
        "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4",
        className
      )}>
        {[...Array(loadingCount)].map((_, i) => (
          <Card key={i} className="h-full animate-pulse">
            <div className="p-6">
              <div className="space-y-3">
                <div className="h-4 bg-muted rounded w-3/4"></div>
                <div className="h-3 bg-muted rounded w-full"></div>
                <div className="h-3 bg-muted rounded w-2/3"></div>
              </div>
              <div className="mt-6 h-20 bg-muted rounded"></div>
            </div>
          </Card>
        ))}
      </div>
    );
  }

  return (
    <div className={cn(
      "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4",
      className
    )}>
      {children}
    </div>
  );
}