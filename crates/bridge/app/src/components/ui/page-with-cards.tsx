"use client";

import React, { ReactNode, useState } from "react";
import { Search, Plus } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { PageHeader } from "@/components/ui/page-header";
import { PageLayout } from "@/components/ui/page-layout";
import { CardGrid } from "@/components/ui/card-grid";
import { EmptyState } from "@/components/ui/empty-state";

interface PageWithCardsProps {
  title: string;
  description?: string;
  searchPlaceholder?: string;
  onSearchChange?: (query: string) => void;
  createButton?: {
    label: string;
    icon?: LucideIcon;
    onClick: () => void;
  };
  emptyState?: {
    icon?: LucideIcon;
    title: string;
    description?: string;
    actionLabel?: string;
    actionIcon?: LucideIcon;
    onAction?: () => void;
  };
  loading?: boolean;
  children: ReactNode;
}

export function PageWithCards({
  title,
  description,
  searchPlaceholder = "Search...",
  onSearchChange,
  createButton,
  emptyState,
  loading = false,
  children
}: PageWithCardsProps) {
  const [searchQuery, setSearchQuery] = useState("");

  const handleSearchChange = (value: string) => {
    setSearchQuery(value);
    onSearchChange?.(value);
  };

  return (
    <div className="min-h-screen bg-background">
      <PageLayout className="py-8">
        <div className="space-y-6">
          {/* Page Header */}
          <div className="space-y-4">
            <PageHeader
              title={title}
              description={description}
            />

            {/* Search and Actions Bar */}
            <div className="flex justify-between">
              <div className="relative flex-1 max-w-md">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder={searchPlaceholder}
                  value={searchQuery}
                  onChange={(e) => handleSearchChange(e.target.value)}
                  className="pl-9"
                />
              </div>
              {createButton && (
                <Button onClick={createButton.onClick}>
                  {createButton.icon && <createButton.icon className="h-4 w-4 mr-2" />}
                  {createButton.label}
                </Button>
              )}
            </div>
          </div>

          {/* Content Area */}
          {loading ? (
            <CardGrid loading={true} />
          ) : emptyState && React.Children.count(children) === 0 ? (
            <EmptyState
              icon={emptyState.icon}
              title={emptyState.title}
              description={emptyState.description}
              action={emptyState.onAction ? {
                label: emptyState.actionLabel || "Get Started",
                icon: emptyState.actionIcon,
                onClick: emptyState.onAction
              } : undefined}
            />
          ) : (
            children
          )}
        </div>
      </PageLayout>
    </div>
  );
}

// Re-export for convenience
export { CardGrid } from "@/components/ui/card-grid";
export { CardItem } from "@/components/ui/card-item";
export { EmptyState } from "@/components/ui/empty-state";