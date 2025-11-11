import { createSomaAgent, patterns } from "@soma/sdk";
// import { DefaultApi as BridgeApi, generatedBridgeClient } from './../.soma/bridge-client'
import { CreateMessageRequest, CreateMessageResponse, MessagePartTypeEnum, MessageRole, DefaultApi as SomaApi, TaskStatus, TaskTimelineItem } from '@soma/api-client'
import { z } from "zod";
import { LanguageModel, LanguageModelMiddleware, ModelMessage, streamText, tool, wrapLanguageModel } from "ai";
import { openai } from "@ai-sdk/openai";
import { durableCalls } from "@restatedev/vercel-ai-middleware";
import { ObjectContext, RestatePromise, } from "@restatedev/restate-sdk";
import { Message } from "../../../packages/api-client/dist/models/Message";
import { convertToAiSdkMessages } from "../utils";
/////

const InsuranceClaimSchema = z.object({
	date: z.string().nonoptional().optional(),
	category: z.string().nonoptional().optional(),
	reason: z.string().nonoptional().optional(),
	amount: z.number().nonoptional().optional(),
	email: z.string().nonoptional().optional(),
});

type InsuranceClaim = z.infer<typeof InsuranceClaimSchema>;
export const assessmentSchema = z.object({
	claim: InsuranceClaimSchema,
});

type Assessment = z.infer<typeof assessmentSchema>

interface BaseHandlerInput {
	model: LanguageModel
}

interface DiscoverClaimInput {
	model: LanguageModel
}

interface ProcessClaimInput {
	assessment: Assessment
}

const handlers = {
	discoverClaim: patterns.chat<any, DiscoverClaimInput, Assessment>(async ({ ctx, bridge, soma, history, input: { model }, onGoalAchieved, sendMessage }) => {
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
					execute: (input) => onGoalAchieved(input)
				}),
			}
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
			parts: [{
				text: agentOutput,
				metadata: {},
				type: MessagePartTypeEnum.TextPart,
			}],
			referenceTaskIds: [],
			role: MessageRole.Agent,
		});
	}),
	processClaim: patterns.workflow<any, ProcessClaimInput, void>(async ({ ctx, bridge, soma, history, input: { assessment }, sendMessage }) => {
		ctx.console.log("Assessment", assessment);


		await sendMessage({
			metadata: {},
			parts: [{
				text: "Please wait while we process your claim... You should receive an email with the results shortly.",
				metadata: {},
				type: MessagePartTypeEnum.TextPart,
			}],
			referenceTaskIds: [],
			role: MessageRole.Agent,
		});


		await ctx.run(async () => await bridge.invokeDanieltrysomaaiGoogleMailSendEmail({
			googleMailgoogleMailSendEmailParamsWrapper: {
				params: {
					body: "Your claim has been processed. Please find the results attached.",
					subject: "Insurance Claim Processed",
					to: assessment.claim.email || ""
				}
			},
		}));
		

		await ctx.run(async () => await bridge.invokeInternalBotApproveClaim({
			approveClaimapproveClaimParamsWrapper: {
				params: {
					claim: assessment.claim,
				}
			},
		}));

	}),
};
let generatedBridgeClient = undefined as any;
export default createSomaAgent({
	generatedBridgeClient,
	projectId: "danielblignaut",
	agentId: "insuranceClaimsAgent",
	name: "Insurance Claims Agent",
	description: "An agent that can process insurance claims.",
	entrypoint: async ({ ctx, soma, bridge, taskId, contextId }) => {
		const model = wrapLanguageModel({
			model: openai("gpt-4o"),
			middleware: durableCalls(ctx, { maxRetryAttempts: 3 }),
		});

		ctx.console.log("Discovering claim...");
		const assessment = await handlers.discoverClaim({
			ctx,
			bridge,
			input: { model },
			taskId,
			soma,
			firstTurn: 'agent',
		});

		await handlers.processClaim({
			ctx,
			bridge,
			input: { assessment },
			taskId,
			soma,
			interruptable: false,
		})

		await ctx.run(() => soma.updateTaskStatus({
			taskId,
			updateTaskStatusRequest: {
				status: TaskStatus.Completed,
				message: {
					metadata: {},
					parts: [{
						metadata: {},
						type: MessagePartTypeEnum.TextPart,
						text: "Claim processed",
					}],
					referenceTaskIds: [],
					role: "agent"
				},
			},
		}));

	},
});

