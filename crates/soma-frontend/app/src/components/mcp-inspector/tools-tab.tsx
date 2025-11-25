import type {
	CompatibilityCallToolResult,
	Tool,
} from "@modelcontextprotocol/sdk/types.js";
import { Send } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { TabsContent } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { JsonView } from "./json-view";

interface ToolsTabProps {
	tools: Tool[];
	listTools: () => void;
	clearTools: () => void;
	callTool: (name: string, params: Record<string, unknown>) => Promise<void>;
	selectedTool: Tool | null;
	setSelectedTool: (tool: Tool | null) => void;
	toolResult: CompatibilityCallToolResult | null;
}

export function ToolsTab({
	tools,
	listTools,
	clearTools,
	callTool,
	selectedTool,
	setSelectedTool,
	toolResult,
}: ToolsTabProps) {
	const [expandedTools, setExpandedTools] = useState<Record<number, boolean>>(
		{},
	);
	const [params, setParams] = useState<Record<number, Record<string, unknown>>>(
		{},
	);
	const [isRunning, setRunningTools] = useState<Record<number, boolean>>({});

	const toggleTool = (index: number, tool: Tool) => {
		setExpandedTools((prev) => {
			const newExpanded = { ...prev, [index]: !prev[index] };
			if (newExpanded[index] && !params[index]) {
				// Initialize params when expanding
				const defaultParams: Record<string, unknown> = {};
				const properties = tool.inputSchema?.properties || {};
				Object.keys(properties).forEach((key) => {
					defaultParams[key] = "";
				});
				setParams((prevParams) => ({ ...prevParams, [index]: defaultParams }));
			}
			return newExpanded;
		});
	};

	const handleRunTool = async (index: number, tool: Tool) => {
		const toolParams = params[index] || {};
		try {
			setRunningTools((prev) => ({ ...prev, [index]: true }));
			await callTool(tool.name, toolParams);
		} finally {
			setRunningTools((prev) => ({ ...prev, [index]: false }));
		}
	};

	return (
		<TabsContent value="tools" className="space-y-4">
			<div>
				<div className="mb-4">
					<h3 className="font-semibold mb-4">Tools</h3>
					<Button variant="outline" className="w-full mb-2" onClick={listTools}>
						List Tools
					</Button>
					<Button
						variant="outline"
						className="w-full mb-4"
						onClick={clearTools}
						disabled={tools.length === 0}
					>
						Clear
					</Button>
				</div>
				{tools.length === 0 ? (
					<p className="text-sm text-muted-foreground italic">
						No tools available. Click "List Tools" to load them.
					</p>
				) : (
					<ul className="space-y-3">
						{tools.map((tool, index) => {
							const isExpanded = expandedTools[index] || false;
							const toolParams = params[index] || {};
							const isToolRunning = isRunning[index] || false;
							const isSelected = selectedTool?.name === tool.name;
							const result = isSelected ? toolResult : null;

							return (
								<li
									key={index}
									className="text-sm bg-white border rounded-lg py-2 px-3 shadow-sm"
								>
									<div
										className="flex justify-between items-center cursor-pointer"
										onClick={() => toggleTool(index, tool)}
									>
										<div className="flex flex-col items-start flex-1">
											<span className="font-medium">{tool.name}</span>
											<span className="text-sm text-muted-foreground line-clamp-2">
												{tool.description}
											</span>
										</div>
										<span className="ml-2">{isExpanded ? "▼" : "▶"}</span>
									</div>
									{isExpanded && (
										<div className="mt-4 space-y-4">
											<div className="mb-2">
												<p className="text-sm text-muted-foreground">
													{tool.description}
												</p>
											</div>

											{Object.entries(tool.inputSchema?.properties || {}).map(
												([key, schema]: [string, any]) => (
													<div key={key} className="space-y-1">
														<Label htmlFor={`${index}-${key}`}>{key}</Label>
														{schema.type === "string" && schema.enum ? (
															<select
																id={`${index}-${key}`}
																className="w-full px-3 py-2 border rounded-md"
																value={String(toolParams[key] || "")}
																onChange={(e) =>
																	setParams({
																		...params,
																		[index]: {
																			...toolParams,
																			[key]: e.target.value,
																		},
																	})
																}
															>
																<option value="">Select...</option>
																{schema.enum.map((option: string) => (
																	<option key={option} value={option}>
																		{option}
																	</option>
																))}
															</select>
														) : schema.type === "string" ? (
															<Textarea
																id={`${index}-${key}`}
																placeholder={schema.description}
																value={String(toolParams[key] || "")}
																onChange={(e) =>
																	setParams({
																		...params,
																		[index]: {
																			...toolParams,
																			[key]: e.target.value,
																		},
																	})
																}
															/>
														) : schema.type === "number" ||
															schema.type === "integer" ? (
															<Input
																type="number"
																id={`${index}-${key}`}
																placeholder={schema.description}
																value={String(toolParams[key] || "")}
																onChange={(e) =>
																	setParams({
																		...params,
																		[index]: {
																			...toolParams,
																			[key]: e.target.value
																				? Number(e.target.value)
																				: undefined,
																		},
																	})
																}
															/>
														) : schema.type === "boolean" ? (
															<div className="flex items-center space-x-2">
																<input
																	type="checkbox"
																	id={`${index}-${key}`}
																	checked={!!toolParams[key]}
																	onChange={(e) =>
																		setParams({
																			...params,
																			[index]: {
																				...toolParams,
																				[key]: e.target.checked,
																			},
																		})
																	}
																/>
																<Label htmlFor={`${index}-${key}`}>
																	{schema.description}
																</Label>
															</div>
														) : (
															<Textarea
																id={`${index}-${key}`}
																placeholder="Enter JSON"
																value={
																	typeof toolParams[key] === "object"
																		? JSON.stringify(toolParams[key], null, 2)
																		: String(toolParams[key] || "")
																}
																onChange={(e) => {
																	try {
																		setParams({
																			...params,
																			[index]: {
																				...toolParams,
																				[key]: JSON.parse(e.target.value),
																			},
																		});
																	} catch {
																		setParams({
																			...params,
																			[index]: {
																				...toolParams,
																				[key]: e.target.value,
																			},
																		});
																	}
																}}
															/>
														)}
													</div>
												),
											)}

											<Button
												onClick={() => {
													setSelectedTool(tool);
													handleRunTool(index, tool);
												}}
												disabled={isToolRunning}
												className="w-full"
											>
												{isToolRunning ? (
													<>Running...</>
												) : (
													<>
														<Send className="w-4 h-4 mr-2" />
														Run Tool
													</>
												)}
											</Button>

											{result && isSelected && (
												<div>
													<h4 className="font-semibold mb-2">Result:</h4>
													<JsonView data={result} isError={!!result.isError} />
												</div>
											)}
										</div>
									)}
								</li>
							);
						})}
					</ul>
				)}
			</div>
		</TabsContent>
	);
}
