import type { TaskTimelineItem } from "@trysoma/api-client/dist/models/TaskTimelineItem";
import type { ModelMessage } from "ai";

export const convertToAiSdkMessages = (messages: TaskTimelineItem[]) => {
	const finalMessages: ModelMessage[] = messages
		.filter((item) => item.eventPayload.type === "message")
		.sort((a, b) => {
			const dateA =
				a.createdAt instanceof Date ? a.createdAt : new Date(a.createdAt);
			const dateB =
				b.createdAt instanceof Date ? b.createdAt : new Date(b.createdAt);
			return dateA.getTime() - dateB.getTime();
		})
		.map((item) => {
			if (item.eventPayload.type === "task-status-update") {
				throw new Error("Task status update is not supported in chat history");
			}

			const text = item.eventPayload.message.parts
				.map((part) => part.text)
				.join("\n");

			if (item.eventPayload.message.role === "agent") {
				return {
					role: "assistant",
					content: text,
				};
			} else {
				return {
					role: "user",
					content: text,
				};
			}
		});

	return finalMessages;
};
