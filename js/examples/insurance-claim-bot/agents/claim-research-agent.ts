import { experimental_createMCPClient as createMCPClient } from "@ai-sdk/mcp";
import { openai } from "@ai-sdk/openai";
import { durableCalls } from "@restatedev/vercel-ai-middleware";
import {
	MessagePartTypeEnum,
	MessageRole,
	TaskStatus,
} from "@trysoma/api-client";
import { createSomaAgent, patterns } from "@trysoma/sdk";
import {
	type LanguageModel,
	streamText,
	type ToolSet,
	tool,
	wrapLanguageModel,
} from "ai";
import { z } from "zod";
import { type AgentsDefinition, getAgents } from "../soma/agents";
import { type BridgeDefinition, getBridge } from "../soma/bridge";
import { convertToAiSdkMessages } from "../utils";
import { createSomaMcpClient, createSomaMcpTransport } from "@trysoma/sdk/mcp";
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

			let {
				transport,
				sessionId,
			} = createSomaMcpTransport(ctx, "test");
			const mcpClient = await createMCPClient({
				transport,
			})
			sessionId = transport.sessionId;
			const tools = await mcpClient.tools();
			ctx.console.log("Tools", tools.tools);

			const stream = streamText({
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
			let agentOutput = "";

			for await (const evt of stream.fullStream) {
				if (evt.type === "text-delta") {
					process.stdout.write(evt.text);
					agentOutput += evt.text;
					// messages.push({ role: "assistant", content: agentOutput });
					// stream output back to user
				}
			}

			await sendMessage({
				metadata: {},
				parts: [
					{
						text: agentOutput,
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

		const model = wrapLanguageModel({
			model: openai("gpt-5"),
			middleware: durableCalls(ctx, { maxRetryAttempts: 3 }),
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
