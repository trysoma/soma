"use client";

import { ChevronDown, ChevronRight } from "lucide-react";
import { useMemo, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

export interface TraceSpan {
	traceId: string;
	spanId: string;
	parentSpanId?: string;
	operationName: string;
	serviceName: string;
	startTime: number; // microseconds
	duration: number; // microseconds
	tags?: Record<string, string | number | boolean>;
	logs?: Array<{
		timestamp: number;
		fields: Record<string, string | number | boolean>;
	}>;
	status?: {
		code: "OK" | "ERROR" | "UNSET";
		message?: string;
	};
	attributes?: Record<string, string | number | boolean>;
	events?: Array<{
		name: string;
		timestamp: number;
		attributes?: Record<string, string | number | boolean>;
	}>;
	kind?: "INTERNAL" | "SERVER" | "CLIENT" | "PRODUCER" | "CONSUMER";
}

interface TraceWaterfallProps {
	spans: TraceSpan[];
	className?: string;
}

interface SpanWithChildren extends TraceSpan {
	children: SpanWithChildren[];
	depth: number;
}

// Helper function to build tree structure from flat spans
function buildSpanTree(spans: TraceSpan[]): SpanWithChildren[] {
	const spanMap = new Map<string, SpanWithChildren>();
	const rootSpans: SpanWithChildren[] = [];

	// First pass: create all span nodes
	spans.forEach((span) => {
		spanMap.set(span.spanId, {
			...span,
			children: [],
			depth: 0,
		});
	});

	// Second pass: build tree structure
	spans.forEach((span) => {
		const currentSpan = spanMap.get(span.spanId);
		if (!currentSpan) return;

		if (span.parentSpanId && spanMap.has(span.parentSpanId)) {
			const parentSpan = spanMap.get(span.parentSpanId);
			if (parentSpan) {
				parentSpan.children.push(currentSpan);
				currentSpan.depth = parentSpan.depth + 1;
			}
		} else {
			rootSpans.push(currentSpan);
		}
	});

	// Sort children by start time
	const sortChildren = (span: SpanWithChildren) => {
		span.children.sort((a, b) => a.startTime - b.startTime);
		span.children.forEach(sortChildren);
	};
	rootSpans.forEach(sortChildren);

	return rootSpans;
}

// Format duration for display
function formatDuration(microseconds: number): string {
	if (microseconds < 1000) {
		return `${microseconds.toFixed(0)}μs`;
	} else if (microseconds < 1000000) {
		return `${(microseconds / 1000).toFixed(2)}ms`;
	} else {
		return `${(microseconds / 1000000).toFixed(2)}s`;
	}
}

// Get span type badge variant and label
function getSpanTypeInfo(span: TraceSpan): {
	variant: "default" | "secondary" | "outline" | "destructive";
	label: string;
} {
	// Check for LLM-specific types from tags/attributes
	const attributes = { ...span.tags, ...span.attributes };
	const spanType =
		attributes?.["span.type"] || attributes?.component || span.kind;

	if (attributes?.["llm.model"] || attributes?.["ai.model"]) {
		return { variant: "default", label: "LLM" };
	}
	if (spanType === "agent" || attributes?.["agent.name"]) {
		return { variant: "secondary", label: "AGENT" };
	}
	if (spanType === "tool" || attributes?.["tool.name"]) {
		return { variant: "outline", label: "TOOL" };
	}
	if (spanType === "retriever" || attributes?.["retriever.name"]) {
		return { variant: "default", label: "RETRIEVER" };
	}
	if (spanType === "embeddings" || attributes?.["embeddings.model"]) {
		return { variant: "secondary", label: "EMBEDDINGS" };
	}
	if (span.status?.code === "ERROR") {
		return { variant: "destructive", label: "ERROR" };
	}

	// Default based on span kind
	switch (span.kind) {
		case "SERVER":
			return { variant: "default", label: "SERVER" };
		case "CLIENT":
			return { variant: "outline", label: "CLIENT" };
		default:
			return { variant: "secondary", label: "SPAN" };
	}
}

interface SpanRowProps {
	span: SpanWithChildren;
	totalDuration: number;
	minStartTime: number;
	onToggle?: (spanId: string) => void;
	isExpanded: boolean;
	rowIndex: number;
}

function SpanRow({
	span,
	totalDuration,
	minStartTime,
	onToggle,
	isExpanded,
	rowIndex,
}: SpanRowProps) {
	const relativeStartTime = span.startTime - minStartTime;
	const startPercentage = (relativeStartTime / totalDuration) * 100;
	const durationPercentage = (span.duration / totalDuration) * 100;
	const typeInfo = getSpanTypeInfo(span);
	const hasChildren = span.children.length > 0;
	const hasError = span.status?.code === "ERROR";

	return (
		<div className="group">
			<div
				className={cn(
					"flex items-center border-0 transition-colors h-8",
					rowIndex % 2 === 0 ? "bg-white" : "bg-gray-50",
					"hover:bg-gray-100",
					hasError && "bg-destructive/5 hover:bg-destructive/10",
				)}
			>
				{/* Service & Operation Name Column */}
				<button
					type="button"
					className="flex items-center gap-2 px-2 py-1 min-w-0 flex-shrink-0 cursor-pointer text-left"
					style={{
						width: "400px",
						paddingLeft: `${span.depth * 20 + 8}px`,
					}}
					onClick={() => hasChildren && onToggle?.(span.spanId)}
				>
					{hasChildren && (
						<span className="flex-shrink-0 p-0.5 hover:bg-muted rounded">
							{isExpanded ? (
								<ChevronDown className="h-3.5 w-3.5" />
							) : (
								<ChevronRight className="h-3.5 w-3.5" />
							)}
						</span>
					)}
					{!hasChildren && <div className="w-5" />}

					<div className="flex items-center gap-2 min-w-0">
						<Badge
							variant={typeInfo.variant}
							className="text-[10px] px-1.5 py-0 h-5 flex-shrink-0 font-mono"
						>
							{typeInfo.label}
						</Badge>
						<span
							className="text-xs font-mono truncate"
							title={span.operationName}
						>
							{span.operationName}
						</span>
					</div>
				</button>

				{/* Duration Column */}
				<div className="w-20 flex-shrink-0 text-right pr-3">
					<span className="text-xs font-mono">
						{formatDuration(span.duration)}
					</span>
				</div>

				{/* Timeline Visualization */}
				<div className="flex-1 relative h-8 px-2">
					<div className="absolute inset-0 flex items-center">
						{/* Grid lines for reference */}
						<div className="absolute inset-0 flex">
							{[0, 25, 50, 75].map((percent) => (
								<div
									key={percent}
									className="absolute top-0 bottom-0 border-l border-border/20"
									style={{ left: `${percent}%` }}
								/>
							))}
						</div>

						{/* Span bar */}
						<div
							className={cn(
								"absolute h-4 rounded-sm border transition-all",
								hasError
									? "bg-destructive/20 border-destructive hover:bg-destructive/30"
									: "bg-primary/20 border-primary/50 hover:bg-primary/30",
							)}
							style={{
								left: `${Math.max(0, Math.min(startPercentage, 100))}%`,
								width: `${Math.max(0.5, Math.min(durationPercentage, 100 - startPercentage))}%`,
							}}
							title={`Start: ${formatDuration(relativeStartTime)}, Duration: ${formatDuration(span.duration)}`}
						/>
					</div>
				</div>
			</div>

			{/* Render children if expanded */}
			{isExpanded && hasChildren && (
				<div>
					{span.children.map((childSpan, index) => (
						<SpanRowWithState
							key={childSpan.spanId}
							span={childSpan}
							totalDuration={totalDuration}
							minStartTime={minStartTime}
							rowIndex={rowIndex + index + 1}
						/>
					))}
				</div>
			)}
		</div>
	);
}

// Wrapper component to manage individual span expansion state
function SpanRowWithState(
	props: Omit<SpanRowProps, "isExpanded" | "onToggle">,
) {
	const [isExpanded, setIsExpanded] = useState(true);

	return (
		<SpanRow
			{...props}
			isExpanded={isExpanded}
			onToggle={() => setIsExpanded(!isExpanded)}
		/>
	);
}

export function TraceWaterfall({ spans, className }: TraceWaterfallProps) {
	const spanTree = useMemo(() => buildSpanTree(spans), [spans]);

	// Calculate total duration and min start time
	const { minStartTime, totalDuration } = useMemo(() => {
		if (spans.length === 0) {
			return { minStartTime: 0, totalDuration: 0 };
		}

		const minStart = Math.min(...spans.map((s) => s.startTime));
		const maxEnd = Math.max(...spans.map((s) => s.startTime + s.duration));
		return {
			minStartTime: minStart,
			totalDuration: maxEnd - minStart,
		};
	}, [spans]);

	if (spans.length === 0) {
		return (
			<div className="flex items-center justify-center p-8 text-muted-foreground">
				No spans to display
			</div>
		);
	}

	return (
		<div className={cn("w-full bg-white  border-y overflow-hidden", className)}>
			{/* Header */}
			<div className="flex items-center border-b bg-gray-100 font-mono font-semibold text-xs h-8">
				<div className="px-3 py-1" style={{ width: "400px" }}>
					Service & Operation
				</div>
				<div className="w-20 text-right pr-3 py-1">Duration</div>
				<div className="flex-1 px-2 py-1">Timeline</div>
			</div>

			{/* Spans */}
			<div className="overflow-auto bg-white">
				{spanTree.map((span, index) => (
					<SpanRowWithState
						key={span.spanId}
						span={span}
						totalDuration={totalDuration}
						minStartTime={minStartTime}
						rowIndex={index}
					/>
				))}
			</div>

			{/* Timeline scale footer */}
			<div className="flex items-center border-t bg-gray-100 text-xs font-mono text-muted-foreground h-8">
				<div style={{ width: "400px" }} />
				<div className="w-20" />
				<div className="flex-1 flex justify-between px-2 py-1">
					<span>0μs</span>
					<span>{formatDuration(totalDuration / 4)}</span>
					<span>{formatDuration(totalDuration / 2)}</span>
					<span>{formatDuration((totalDuration * 3) / 4)}</span>
					<span>{formatDuration(totalDuration)}</span>
				</div>
			</div>
		</div>
	);
}
