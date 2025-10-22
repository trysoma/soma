"use client"

import * as React from "react"

import { cn } from "@/lib/utils"

// Wrapper component for table with border and background
function TableWrapper({
  className,
  children
}: {
  className?: string;
  children: React.ReactNode
}) {
  return (
    <div className={cn("border rounded-lg bg-white", className)}>
      {children}
    </div>
  )
}

// Table title section
function TableTitle({
  className,
  children
}: {
  className?: string;
  children: React.ReactNode
}) {
  return (
    <div className={cn("px-4 py-3 border-b", className)}>
      <h3 className="font-semibold text-sm">{children}</h3>
    </div>
  )
}

// Scrollable container for the table
function TableContainer({
  className,
  maxHeight = "max-h-[600px]",
  children
}: {
  className?: string;
  maxHeight?: string;
  children: React.ReactNode
}) {
  return (
    <div className={cn("overflow-auto", maxHeight, className)}>
      {children}
    </div>
  )
}

// Empty state for tables
function TableEmpty({
  className,
  children
}: {
  className?: string;
  children: React.ReactNode
}) {
  return (
    <div className={cn("p-8 text-center text-sm text-muted-foreground", className)}>
      {children}
    </div>
  )
}

// Loading more indicator
function TableLoadMore({
  className,
  loadMoreRef
}: {
  className?: string;
  loadMoreRef?: React.RefObject<HTMLDivElement | null>;
}) {
  return (
    <div
      ref={loadMoreRef}
      className={cn("p-4 text-center text-sm text-muted-foreground", className)}
    >
      Loading more...
    </div>
  )
}

function Table({ className, ...props }: React.ComponentProps<"table">) {
  return (
    <div className="relative w-full overflow-x-auto">
      <table
        className={cn("w-full caption-bottom text-sm", className)}
        {...props}
      />
    </div>
  )
}

function TableHeader({
  className,
  sticky = false,
  ...props
}: React.ComponentProps<"thead"> & { sticky?: boolean }) {
  return (
    <thead
      className={cn(
        "[&_tr]:border-b",
        sticky && "sticky top-0 z-10 bg-white",
        className
      )}
      {...props}
    />
  )
}

function TableBody({ className, ...props }: React.ComponentProps<"tbody">) {
  return (
    <tbody
      className={cn("[&_tr:last-child]:border-0", className)}
      {...props}
    />
  )
}

function TableFooter({ className, ...props }: React.ComponentProps<"tfoot">) {
  return (
    <tfoot
      className={cn(
        "bg-muted/50 border-t font-medium [&>tr]:last:border-b-0",
        className
      )}
      {...props}
    />
  )
}

function TableRow({
  className,
  index,
  ...props
}: React.ComponentProps<"tr"> & { index?: number }) {
  return (
    <tr
      className={cn(
        "border-b transition-colors hover:bg-gray-50 data-[state=selected]:bg-muted",
        index !== undefined && index % 2 === 1 && "bg-gray-50/50",
        props.onClick && "cursor-pointer",
        // Keep rounded corners on last row when hovering
        "last:[&>td:first-child]:rounded-bl-lg last:[&>td:last-child]:rounded-br-lg",
        className
      )}
      {...props}
    />
  )
}

function TableHead({ className, ...props }: React.ComponentProps<"th">) {
  return (
    <th
      className={cn(
        "h-10 px-3 text-left align-middle font-medium text-xs whitespace-nowrap bg-gray-50 [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]",
        className
      )}
      {...props}
    />
  )
}

function TableCell({
  className,
  bold = false,
  ...props
}: React.ComponentProps<"td"> & { bold?: boolean }) {
  return (
    <td
      className={cn(
        "px-3 py-2 align-middle text-sm whitespace-nowrap [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]",
        bold && "font-medium",
        className
      )}
      {...props}
    />
  )
}

function TableCaption({
  className,
  ...props
}: React.ComponentProps<"caption">) {
  return (
    <caption
      className={cn("text-muted-foreground mt-4 text-sm", className)}
      {...props}
    />
  )
}

export {
  Table,
  TableHeader,
  TableBody,
  TableFooter,
  TableHead,
  TableRow,
  TableCell,
  TableCaption,
  TableWrapper,
  TableTitle,
  TableContainer,
  TableEmpty,
  TableLoadMore,
}
