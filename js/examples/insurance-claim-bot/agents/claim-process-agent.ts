import { openai } from "@ai-sdk/openai";
import { durableCalls } from "@restatedev/vercel-ai-middleware";
import {
	MessagePartTypeEnum,
	MessageRole,
	TaskStatus,
} from "@trysoma/api-client";
import { createSomaAgent, patterns } from "@trysoma/sdk";
import {
	generateText,
	type LanguageModel,
	type ModelMessage,
	streamText,
	tool,
	wrapLanguageModel,
} from "ai";
import { z } from "zod";
import { type AgentsDefinition, getAgents } from "../soma/agents";
import { type BridgeDefinition, getBridge } from "../soma/bridge";
import { convertToAiSdkMessages } from "../utils";

const InsuranceClaimSchema = z.object({
	date: z.string(),
	category: z.string(),
	reason: z.string(),
	amount: z.number(),
	email: z.string(),
});

export const assessmentSchema = z.object({
	claim: InsuranceClaimSchema,
});

type Assessment = z.infer<typeof assessmentSchema>;

interface DiscoverClaimInput {
	model: LanguageModel;
}

interface ProcessClaimInput {
	assessment: Assessment;
	model: LanguageModel;
}

const handlers = {
	discoverClaim: patterns.chat<
		BridgeDefinition,
		AgentsDefinition,
		DiscoverClaimInput,
		Assessment
	>(
		async ({
			ctx,
			soma: _soma,
			history,
			input: { model },
			onGoalAchieved,
			sendMessage,
			bridge: _bridge,
		}) => {
			const messages = convertToAiSdkMessages(history);

			ctx.console.log("Messages", messages);

			const stream = streamText({
				model,
				messages,
				//   abortSignal,
				tools: {
					decodeClaim: tool({
						description: "Decode a claim into a structured object. ",
						inputSchema: assessmentSchema,
						execute: async (input: Assessment) => {
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
	processClaim: patterns.workflow<
		BridgeDefinition,
		AgentsDefinition,
		ProcessClaimInput,
		void
	>(
		async ({
			ctx,
			soma: _soma,
			history,
			bridge,
			input: { assessment, model },
			agents,
			sendMessage,
		}) => {
			ctx.console.log("Beginning to research claim and process it", assessment);

			const messagesWithResearchAgent: ModelMessage[] = [
				...convertToAiSdkMessages(history),
				{
					role: "system",
					content:
						"You are now trying to research familiar claims to the one provided. You are discussing this with a research agent. Any messages / questions you respond with will go to the research agent. include the assessment information in your first message. Once you have some research, call the onComplete tool with an approval decision.",
				},
			];

			let approvalDecision: boolean | undefined;
			while (approvalDecision === undefined) {
				const { text } = await generateText({
					model,
					messages: messagesWithResearchAgent,
					tools: {
						onComplete: tool({
							description: "Call this tool once you have some research",
							inputSchema: z.object({
								approval: z.boolean(),
							}),
							execute: async (input: { approval: boolean }) => {
								approvalDecision = input.approval;
							},
						}),
					},
				});

				const responseStream =
					await agents.acme.claimResearchAgent.sendMessageStream({
						message: {
							role: "user",
							parts: [
								{
									text,
									metadata: {},
									kind: "text",
								},
							],
							messageId: "123",
							kind: "message",
						},
					});

				for await (const event of responseStream) {
					if (event.kind === "message") {
						messagesWithResearchAgent.push({
							role: "user",
							content: event.parts
								.map((part) => (part.kind === "text" ? part.text : ""))
								.join(""),
						});
						break;
					}
				}
			}

			await sendMessage({
				metadata: {},
				parts: [
					{
						text: `Please wait while we process your claim... You should receive an email with the results shortly. Your claim has been ${approvalDecision ? "approved" : "denied"}`,
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
	agentId: "insuranceClaimsAgent",
	name: "Insurance Claims Agent",
	description: "An agent that can process insurance claims.",
	entrypoint: async ({ ctx, soma, taskId, contextId: _contextId }) => {
		const bridge = getBridge(ctx);
		const agents = await getAgents(ctx);

		const model = wrapLanguageModel({
			model: openai("gpt-4o"),
			middleware: durableCalls(ctx, { maxRetryAttempts: 3 }),
		});

		ctx.console.log("Discovering claim...");
		const assessment = await handlers.discoverClaim({
			ctx,
			bridge,
			agents,
			input: { model },
			taskId,
			soma,
			firstTurn: "agent",
		});

		await handlers.processClaim({
			ctx,
			bridge,
			agents,
			input: { assessment, model },
			taskId,
			soma,
			interruptable: false,
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
								text: "Claim processed",
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
