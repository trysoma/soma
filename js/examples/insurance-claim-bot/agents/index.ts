import { openai } from "@ai-sdk/openai";
import { durableCalls } from "@restatedev/vercel-ai-middleware";
import {
	MessagePartTypeEnum,
	MessageRole,
	TaskStatus,
} from "@trysoma/api-client";
import { createSomaAgent, patterns } from "@trysoma/sdk";
import { type LanguageModel, streamText, tool, wrapLanguageModel } from "ai";
import { z } from "zod";
import { convertToAiSdkMessages } from "../utils";

// import { bridge } from "./../.soma/bridge-client";
/////

const InsuranceClaimSchema = z.object({
	date: z.string(),
	category: z.string(),
	reason: z.string(),
	amount: z.number(),
	email: z.string(),
});

// type InsuranceClaim = z.infer<typeof InsuranceClaimSchema>;
export const assessmentSchema = z.object({
	claim: InsuranceClaimSchema,
});

type Assessment = z.infer<typeof assessmentSchema>;

// interface BaseHandlerInput {
// 	model: LanguageModel;
// }

interface DiscoverClaimInput {
	model: LanguageModel;
}

interface ProcessClaimInput {
	assessment: Assessment;
}

const handlers = {
	discoverClaim: patterns.chat<DiscoverClaimInput, Assessment>(
		async ({
			ctx,
			soma: _soma,
			history,
			input: { model },
			onGoalAchieved,
			sendMessage,
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
	processClaim: patterns.workflow<ProcessClaimInput, void>(
		async ({
			ctx,
			soma: _soma,
			history: _history,
			input: { assessment },
			sendMessage,
		}) => {
			ctx.console.log("Assessment", assessment);
			// const b = bridge();

			await sendMessage({
				metadata: {},
				parts: [
					{
						text: "Please wait while we process your claim... You should receive an email with the results shortly.",
						metadata: {},
						type: MessagePartTypeEnum.TextPart,
					},
				],
				referenceTaskIds: [],
				role: MessageRole.Agent,
			});

			// await ctx.run(async () => await b.invokeDanieltrysomaaiGoogleMailSendEmail({
			// 	googleMailgoogleMailSendEmailParamsWrapper: {
			// 		params: {
			// 			body: "Your claim has been processed. Please find the results attached.",
			// 			subject: "Insurance Claim Processed",
			// 			to: assessment.claim.email || ""
			// 		}
			// 	},
			// }));

			// await ctx.run(async () => await b.invokeInternalBotApproveClaim({
			// 	approveClaimapproveClaimParamsWrapper: {
			// 		params: {
			// 			claim: assessment.claim,
			// 		}
			// 	},
			// }));
		},
	),
};
export default createSomaAgent({
	projectId: "danielblignaut",
	agentId: "insuranceClaimsAgent",
	name: "Insurance Claims Agent",
	description: "An agent that can process insurance claims.",
	entrypoint: async ({ ctx, soma, taskId, contextId: _contextId }) => {
		const model = wrapLanguageModel({
			model: openai("gpt-4o"),
			middleware: durableCalls(ctx, { maxRetryAttempts: 3 }),
		});

		ctx.console.log("Discovering claim...");
		const assessment = await handlers.discoverClaim({
			ctx,
			input: { model },
			taskId,
			soma,
			firstTurn: "agent",
		});

		await handlers.processClaim({
			ctx,
			input: { assessment },
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
