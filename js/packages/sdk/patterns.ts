import { type ObjectContext, RestatePromise } from "@restatedev/restate-sdk";
import type {
	CreateMessageRequest,
	CreateMessageResponse,
	DefaultApi,
	TaskTimelineItem,
} from "@trysoma/api-client";

export type FirstTurn = "user" | "agent";

export interface WrappedChatHandlerParams<Bridge, Input, _Output> {
	ctx: ObjectContext;
	soma: DefaultApi;
	bridge: Bridge;
	input: Input;
	taskId: string;
	firstTurn: FirstTurn;
}

export interface ChatHandlerParams<Bridge, Input, Output> {
	ctx: ObjectContext;
	soma: DefaultApi;
	bridge: Bridge;
	history: TaskTimelineItem[];
	input: Input;
	onGoalAchieved: (goal: Output) => void;
	sendMessage: (
		message: CreateMessageRequest,
	) => RestatePromise<CreateMessageResponse>;
}

type Goal<Output> =
	| {
			type: "achieved";
			output: Output;
	  }
	| {
			type: "not_achieved";
	  };

export const chat = <Bridge, Input, Output>(
	handler: (params: ChatHandlerParams<Bridge, Input, Output>) => Promise<void>,
) => {
	return async ({
		ctx,
		soma,
		bridge,
		input,
		taskId,
		firstTurn = "user",
	}: WrappedChatHandlerParams<Bridge, Input, Output>) => {
		const NEW_INPUT_PROMISE = `new_input_promise`;
		let { id: awakeableId, promise: newInputPromise } =
			await ctx.awakeable<void>();
		await ctx.set(NEW_INPUT_PROMISE, awakeableId);

		let goal: Goal<Output> = { type: "not_achieved" };
		const onGoalAchieved = (output: Output) => {
			goal = { type: "achieved", output };
			achieved = true;
		};
		let achieved = false;
		if (firstTurn === "user") {
			await newInputPromise;
		}
		while (!achieved) {
			// new message will be in the history already
			const messages = await ctx.run(
				async () =>
					await soma.taskHistory({
						pageSize: 1000,
						taskId,
					}),
			);

			const sendMessage = (message: CreateMessageRequest) => {
				return ctx.run(
					async () =>
						await soma.sendMessage({
							taskId,
							createMessageRequest: message,
						}),
				);
			};

			await handler({
				ctx,
				soma,
				history: messages.items,
				bridge,
				input,
				onGoalAchieved,
				sendMessage,
			});
			if (goal.type === "not_achieved") {
				// re-arm the awakeable, waiting for another message
				const { id: newId, promise: nextPromise } = await ctx.awakeable<void>();
				await ctx.set(NEW_INPUT_PROMISE, newId);
				newInputPromise = nextPromise;

				await newInputPromise;
			} else {
				break;
			}
		}

		if (goal.type === "not_achieved") {
			throw new Error("Goal not achieved");
		} else {
			return goal.output as Output;
		}
	};
};

export interface WorkflowHandlerParams<Bridge, Input, _Output> {
	ctx: ObjectContext;
	soma: DefaultApi;
	bridge: Bridge;
	history: TaskTimelineItem[];
	input: Input;
	sendMessage: (
		message: CreateMessageRequest,
	) => RestatePromise<CreateMessageResponse>;
	interruptable: boolean;
}

export interface WrappedWorkflowHandlerParams<Bridge, Input, _Output> {
	ctx: ObjectContext;
	soma: DefaultApi;
	bridge: Bridge;
	input: Input;
	taskId: string;
	interruptable?: boolean;
}

export const workflow = <Bridge, Input, Output>(
	handler: (params: WorkflowHandlerParams<Bridge, Input, Output>) => Promise<Output>,
) => {
	return async ({
		ctx,
		soma,
		bridge,
		input,
		taskId,
		interruptable = true,
	}: WrappedWorkflowHandlerParams<Bridge, Input, Output>) => {
		while (true) {
			const NEW_INPUT_PROMISE = `new_input_promise`;
			const { id: awakeableId, promise: newInputPromise } =
				await ctx.awakeable<void>();
			await ctx.set(NEW_INPUT_PROMISE, awakeableId);

			// new message will be in the history already
			const messages = await ctx.run(
				async () =>
					await soma.taskHistory({
						pageSize: 1000,
						taskId,
					}),
			);

			const sendMessage = (message: CreateMessageRequest) => {
				return ctx.run(
					async () =>
						await soma.sendMessage({
							taskId,
							createMessageRequest: message,
						}),
				);
			};

			const handlerPromise = ctx.run(() =>
				handler({
					ctx,
					soma,
					history: messages.items,
					bridge,
					input,
					sendMessage,
					interruptable,
				}),
			);

			if (interruptable) {
				// Race between new input and handler completion
				const raceResult = await RestatePromise.race([
					newInputPromise.map(() => ({ type: "new_input" as const })),
					handlerPromise.map((output) => ({
						type: "handler_complete" as const,
						output,
					})),
				]);

				if (raceResult.type === "new_input") {
				} else {
					// Handler completed first, return the result
					return raceResult.output;
				}
			} else {
				// Not interruptable, just wait for handler to complete
				return await handlerPromise;
			}
		}
	};
};

export const patterns = {
	chat,
	workflow,
};
