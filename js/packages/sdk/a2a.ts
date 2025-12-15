/**
 * Durable A2A client wrapper that makes all A2A operations replayable via Restate.
 * All methods are wrapped with ctx.run() for durability.
 */

import type {
	AgentCard,
	CancelTaskResponse,
	GetTaskResponse,
	Message,
	MessageSendParams,
	SendMessageResponse,
	Task,
	TaskArtifactUpdateEvent,
	TaskIdParams,
	TaskQueryParams,
	TaskStatusUpdateEvent,
} from "@a2a-js/sdk";
import { A2AClient as BaseA2AClient } from "@a2a-js/sdk/client";
import type { ObjectContext } from "@restatedev/restate-sdk";

export type A2AStreamEventData =
	| Message
	| Task
	| TaskStatusUpdateEvent
	| TaskArtifactUpdateEvent;

class DurableStream<T> {
	private stream: AsyncGenerator<T, void, undefined> | undefined;
	private index: number;
	private streamName: string;
	private ctx: ObjectContext;

	constructor(
		ctx: ObjectContext,
		streamName: string,
		stream: AsyncGenerator<T, void, undefined>,
	) {
		this.ctx = ctx;
		this.streamName = streamName;
		this.stream = stream;
		this.index = 0;
	}

	async *[Symbol.asyncIterator]() {
		while (true) {
			const event = await this.ctx.run(
				`${this.streamName}-event-index-${this.index}`,
				async () => this.stream?.next(),
			);

			// Note: this should never happen, this is only
			// if the we initially crashed and now are resuming
			if (!event) {
				throw new Error(
					`Restated durable stream ${this.streamName} error. We should never get here. Our journal is out of sync.`,
				);
			}

			if (event.done) {
				break;
			}
			yield event.value;
			this.index++;
		}
	}
}

/**
 * Durable A2A client wrapper that makes all A2A operations replayable via Restate.
 * All methods are wrapped with ctx.run() for durability.
 */
export class A2AClient {
	private ctx: ObjectContext;
	private client: BaseA2AClient;
	private restateId: string;
	private requestIndex: number;

	constructor(ctx: ObjectContext, client: BaseA2AClient, restateId: string) {
		this.ctx = ctx;
		this.client = client;
		this.restateId = restateId;
		this.requestIndex = 0;
	}

	/**
	 * Send a message to the agent (non-streaming, durable).
	 * The entire request/response cycle is wrapped for durability.
	 */
	async sendMessage(params: MessageSendParams): Promise<SendMessageResponse> {
		return this.ctx.run(
			`a2a-${this.restateId}-sendMessage-index-${this.requestIndex++}`,
			async () => {
				return this.client.sendMessage(params);
			},
		);
	}

	/**
	 * Send a message and stream responses.
	 * Each event from the stream is individually made durable.
	 * Returns an async generator that yields events durably.
	 */
	async *sendMessageStream(
		params: MessageSendParams,
	): AsyncGenerator<A2AStreamEventData, void, undefined> {
		const stream = this.client.sendMessageStream(params);
		const durableStream = new DurableStream(
			this.ctx,
			`a2a-${this.restateId}-sendMessageStream-index-${this.requestIndex++}`,
			stream,
		);
		yield* durableStream;
	}

	/**
	 * Get a task by ID (durable).
	 */
	async getTask(params: TaskQueryParams): Promise<GetTaskResponse> {
		return this.ctx.run(
			`a2a-${this.restateId}-getTask-index-${this.requestIndex++}`,
			async () => {
				return this.client.getTask(params);
			},
		);
	}

	/**
	 * Cancel a task by ID (durable).
	 */
	async cancelTask(params: TaskIdParams): Promise<CancelTaskResponse> {
		return this.ctx.run(
			`a2a-${this.restateId}-cancelTask-index-${this.requestIndex++}`,
			async () => {
				return this.client.cancelTask(params);
			},
		);
	}

	/**
	 * Resubscribe to a task's event stream.
	 * Each event from the stream is individually made durable.
	 */
	async *resubscribeTask(
		params: TaskIdParams,
	): AsyncGenerator<A2AStreamEventData, void, undefined> {
		const stream = this.client.resubscribeTask(params);
		const durableStream = new DurableStream(
			this.ctx,
			`a2a-${this.restateId}-resubscribeTask-index-${this.requestIndex++}`,
			stream,
		);
		yield* durableStream;
	}

	/**
	 * Get the agent card (durable).
	 */
	async getAgentCard(): Promise<AgentCard> {
		return this.ctx.run(
			`a2a-${this.restateId}-getAgentCard-index-${this.requestIndex++}`,
			async () => {
				return this.client.getAgentCard();
			},
		);
	}
}

/**
 * Create a durable A2A client from a card URL.
 *
 * @param ctx - The Restate ObjectContext for durability
 * @param cardUrl - The URL to the agent's card.json
 * @param agentId - A unique identifier for this agent (used for durability keys)
 * @returns A durable A2A client instance
 */
export async function createA2AClient(
	ctx: ObjectContext,
	cardUrl: string,
	agentId: string,
): Promise<A2AClient> {
	const baseClient = await ctx.run(`init-a2a-${agentId}`, async () => {
		return BaseA2AClient.fromCardUrl(cardUrl);
	});
	return new A2AClient(ctx, baseClient, agentId);
}

// Re-export types from @a2a-js/sdk for convenience
export type {
	AgentCard,
	MessageSendParams,
	SendMessageResponse,
	TaskQueryParams,
	GetTaskResponse,
	TaskIdParams,
	CancelTaskResponse,
	Message,
	Task,
	TaskStatusUpdateEvent,
	TaskArtifactUpdateEvent,
};

// Re-export the base client for advanced use cases
export { BaseA2AClient };
