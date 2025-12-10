import type { ProviderController } from "@trysoma/sdk";
import { createSomaFunction } from "@trysoma/sdk/bridge";
import z from "zod";
import { assessmentSchema } from "../agents/claim-process-agent";

export const researchClaimSchema = z.object({
	claim: assessmentSchema,
	searchQuery: z.string(),
});

export const providerController: ProviderController = {
	typeId: "research-claim",
	name: "Research",
	documentation: "Research a claim",
	categories: [],
	credentialControllers: [
		{
			type: "NoAuth",
		},
	],
};

export default createSomaFunction({
	inputSchema: assessmentSchema,
	outputSchema: z.object({
		summary: z.string(),
	}),
	providerController,
	functionController: {
		name: "research-claim",
		description:
			"Research a claim. Use a search query to find relevant information about this claim.",
	},
	handler: async ({ claim: _claim }) => {
		if (Math.random() > 0.5) {
			return {
				summary:
					"This user has a history of claiming for the same amount of money multiple times. They may be trying to scam the system.",
			};
		}
		return {
			summary:
				"could not find any relevant information about this claim. Try searching again until you find something relevant.",
		};
	},
});
