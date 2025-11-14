import type { ServerNotification } from "@modelcontextprotocol/sdk/types.js";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { JsonView } from "./json-view";

interface RequestHistoryItem {
	request: string;
	response?: string;
}

interface HistoryAndNotificationsProps {
	requestHistory: RequestHistoryItem[];
	serverNotifications: ServerNotification[];
	onClearHistory?: () => void;
	onClearNotifications?: () => void;
}

export function HistoryAndNotifications({
	requestHistory,
	serverNotifications,
	onClearHistory,
	onClearNotifications,
}: HistoryAndNotificationsProps) {
	const [expandedRequests, setExpandedRequests] = useState<
		Record<number, boolean>
	>({});
	const [expandedNotifications, setExpandedNotifications] = useState<
		Record<number, boolean>
	>({});
	const [activeTab, setActiveTab] = useState("history");

	const toggleRequest = (index: number) => {
		setExpandedRequests((prev) => ({ ...prev, [index]: !prev[index] }));
	};

	const toggleNotification = (index: number) => {
		setExpandedNotifications((prev) => ({ ...prev, [index]: !prev[index] }));
	};

	return (
		<div className="p-4">
			<Tabs
				value={activeTab}
				onValueChange={setActiveTab}
				className="flex flex-col"
			>
				<TabsList className="mb-4">
					<TabsTrigger value="history">Request History</TabsTrigger>
					<TabsTrigger value="notifications">Server Notifications</TabsTrigger>
				</TabsList>

				<TabsContent value="history" className="space-y-4 m-0">
					<div className="mb-4 flex justify-between items-center">
						<h3 className="font-semibold">Request History</h3>
						<Button
							variant="outline"
							size="sm"
							onClick={onClearHistory}
							disabled={requestHistory.length === 0}
						>
							Clear
						</Button>
					</div>
					{requestHistory.length === 0 ? (
						<p className="text-sm text-muted-foreground italic">
							No history yet
						</p>
					) : (
						<ul className="space-y-3">
							{requestHistory
								.slice()
								.reverse()
								.map((item, index) => {
									const actualIndex = requestHistory.length - 1 - index;
									// Check if response has an error
									let isErrorResponse = false;
									if (item.response) {
										try {
											const parsedResponse = JSON.parse(item.response);
											isErrorResponse = parsedResponse.error !== undefined;
										} catch {
											// If parsing fails, treat as error
											isErrorResponse = true;
										}
									}
									return (
										<li
											key={index}
											className="text-sm bg-white border rounded-lg py-2 px-3 shadow-sm"
										>
											<div
												className="flex justify-between items-center cursor-pointer"
												onClick={() => toggleRequest(actualIndex)}
											>
												<span className="font-mono">
													{requestHistory.length - index}.{" "}
													{JSON.parse(item.request).method}
												</span>
												<span>{expandedRequests[actualIndex] ? "▼" : "▶"}</span>
											</div>
											{expandedRequests[actualIndex] && (
												<>
													<div className="mt-2">
														<span className="text-blue-600">Request:</span>
														<div className="mt-1">
															<JsonView
																data={item.request}
																className="border-blue-500 bg-blue-50"
															/>
														</div>
													</div>
													{item.response && (
														<div className="mt-2">
															<span
																className={
																	isErrorResponse
																		? "text-red-600"
																		: "text-green-600"
																}
															>
																Response:
															</span>
															<div className="mt-1">
																<JsonView
																	data={item.response}
																	className={
																		isErrorResponse
																			? "border-red-500 bg-red-50"
																			: "border-green-500 bg-green-50"
																	}
																	isError={isErrorResponse}
																/>
															</div>
														</div>
													)}
												</>
											)}
										</li>
									);
								})}
						</ul>
					)}
				</TabsContent>

				<TabsContent value="notifications" className="space-y-4 m-0">
					<div className="mb-4 flex justify-between items-center">
						<h3 className="font-semibold">Server Notifications</h3>
						<Button
							variant="outline"
							size="sm"
							onClick={onClearNotifications}
							disabled={serverNotifications.length === 0}
						>
							Clear
						</Button>
					</div>
					{serverNotifications.length === 0 ? (
						<p className="text-sm text-muted-foreground italic">
							No notifications yet
						</p>
					) : (
						<ul className="space-y-3">
							{serverNotifications
								.slice()
								.reverse()
								.map((notification, index) => {
									const actualIndex = serverNotifications.length - 1 - index;
									return (
										<li
											key={index}
											className="text-sm bg-secondary py-2 px-3 rounded"
										>
											<div
												className="flex justify-between items-center cursor-pointer"
												onClick={() => toggleNotification(actualIndex)}
											>
												<span className="font-mono">
													{serverNotifications.length - index}.{" "}
													{notification.method}
												</span>
												<span>
													{expandedNotifications[actualIndex] ? "▼" : "▶"}
												</span>
											</div>
											{expandedNotifications[actualIndex] && (
												<div className="mt-2">
													<JsonView data={notification} />
												</div>
											)}
										</li>
									);
								})}
						</ul>
					)}
				</TabsContent>
			</Tabs>
		</div>
	);
}
