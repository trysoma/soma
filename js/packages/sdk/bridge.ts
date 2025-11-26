import type { FunctionController, ProviderController } from "@trysoma/sdk-core";
import z from "zod";

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
	console.log(z.toJSONSchema(params.inputSchema));
	console.log(z.toJSONSchema(params.outputSchema));
	return {
		inputSchema: params.inputSchema,
		outputSchema: params.outputSchema,
		handler: params.handler,
		providerController: params.providerController,
		functionController: {
			...params.functionController,
			parameters: JSON.stringify(z.toJSONSchema(params.inputSchema)),
			output: JSON.stringify(z.toJSONSchema(params.outputSchema)),
		},
	};
}
