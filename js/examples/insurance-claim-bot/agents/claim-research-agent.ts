import { openai } from "@ai-sdk/openai";
import { durableCalls } from "@restatedev/vercel-ai-middleware";
import {
	MessagePartTypeEnum,
	MessageRole,
	TaskStatus,
} from "@trysoma/api-client";
import { createSomaAgent, patterns } from "@trysoma/sdk";
import { createSomaAiSdkMcpClient } from "@trysoma/sdk/vercel-ai-sdk";
import { type LanguageModel, streamText, tool, wrapLanguageModel } from "ai";
import { z } from "zod";
import { type AgentsDefinition, getAgents } from "../soma/agents";
import { type BridgeDefinition, getBridge } from "../soma/bridge";
import { convertToAiSdkMessages } from "../utils";

interface ClaimResearchInput {
	model: LanguageModel;
}

export const outputResearchSchema = z.object({
	summary: z.string(),
});

type ClaimResearchOutput = z.infer<typeof outputResearchSchema>;

const handlers = {
	claimResearch: patterns.chat<
		BridgeDefinition,
		AgentsDefinition,
		ClaimResearchInput,
		ClaimResearchOutput
	>(
		async ({
			ctx,
			soma: _soma,
			history,
			input: { model },
			onGoalAchieved,
			sendMessage,
			bridge: _bridge,
			agents: _agents,
		}) => {
			const messages = convertToAiSdkMessages(history);
			messages.unshift({
				role: "system",
				content:
					"You are a research agent that can research insurance claims. You are given a claim and you need to research it and return a summary of the research.",
			});

			ctx.console.log("Messages", messages);

			const mcpClient = await createSomaAiSdkMcpClient(ctx, {
				baseMcpClient: {
					mcpServerInstanceId: "test",
				},
			});
			const tools = await mcpClient.tools();
			ctx.console.log("Tools", tools);

			const { fullStream, text } = streamText({
				model,
				messages,
				tools: {
					...tools,
					outputResearch: tool({
						description: "Summarize your findings into a final output ",
						inputSchema: outputResearchSchema,
						execute: async (input: ClaimResearchOutput) => {
							return onGoalAchieved(input);
						},
					}),
				},
			});
			let _agentOutput = "";

			for await (const evt of fullStream) {
				if (evt.type === "text-delta") {
					process.stdout.write(evt.text);
					_agentOutput += evt.text;
					// stream output back to user
				} else if (evt.type === "tool-call") {
					// Tool call initiated - you can log this if needed
					ctx.console.log(`Tool called: ${evt.toolName}`, evt);
				} else if (evt.type === "tool-result") {
					// Tool execution completed - result is available here
					ctx.console.log(`Tool result for ${evt.toolCallId}:`, evt);
					// The model will automatically receive this result and continue streaming
				} else {
					ctx.console.log("Event", evt);
				}
			}
			const finalText = await text;
			ctx.console.log("Final text", finalText);
			messages.push({
				role: "assistant",
				content: finalText,
			});

			await sendMessage({
				metadata: {},
				parts: [
					{
						text: finalText,
						metadata: {},
						type: MessagePartTypeEnum.TextPart,
					},
				],
				referenceTaskIds: [],
				role: MessageRole.Agent,
			});
		},
	),
};
export default createSomaAgent({
	projectId: "acme",
	agentId: "claimResearchAgent",
	name: "Claim Research Agent",
	description: "An agent that can research insurance claims.",
	entrypoint: async ({ ctx, soma, taskId, contextId: _contextId }) => {
		ctx.console.log("Claim Research Agent started");

		const bridge = getBridge(ctx);
		const agents = await getAgents(ctx);

		// Note: Type assertion needed due to version mismatch between ai SDK v3 and restate middleware v2
		const model = wrapLanguageModel({
			model: openai("gpt-5") as unknown as Parameters<
				typeof wrapLanguageModel
			>[0]["model"],
			middleware: durableCalls(ctx, {
				maxRetryAttempts: 3,
			}) as unknown as Parameters<typeof wrapLanguageModel>[0]["middleware"],
		});

		ctx.console.log("Researching claim...");
		const research = await handlers.claimResearch({
			ctx,
			bridge,
			agents,
			input: { model },
			taskId,
			soma,
			firstTurn: "agent",
		});

		await ctx.run(() =>
			soma.updateTaskStatus({
				taskId,
				updateTaskStatusRequest: {
					status: TaskStatus.Completed,
					message: {
						metadata: {},
						parts: [
							{
								metadata: {},
								type: MessagePartTypeEnum.TextPart,
								text: research.summary,
							},
						],
						referenceTaskIds: [],
						role: "agent",
					},
				},
			}),
		);
	},
});
