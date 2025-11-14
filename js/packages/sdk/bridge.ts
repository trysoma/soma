import type { FunctionController, ProviderController } from "@trysoma/sdk-core";
import type z from "zod";
import { zodToJsonSchema } from "zod-to-json-schema";

interface CreateSomaFunctionParams<InputType, OutputType> {
	inputSchema: z.ZodSchema<InputType>;
	outputSchema: z.ZodSchema<OutputType>;
	providerController: ProviderController;
	functionController: Omit<FunctionController, "parameters" | "output">;
	handler: (input: InputType) => Promise<OutputType>;
}

export function createSomaFunction<InputType, OutputType>(
	params: CreateSomaFunctionParams<InputType, OutputType>,
) {
	return {
		inputSchema: params.inputSchema,
		outputSchema: params.outputSchema,
		handler: params.handler,
		providerController: params.providerController,
		functionController: {
			...params.functionController,
			parameters: JSON.stringify(zodToJsonSchema(params.inputSchema)),
			output: JSON.stringify(zodToJsonSchema(params.outputSchema)),
		},
	};
}
