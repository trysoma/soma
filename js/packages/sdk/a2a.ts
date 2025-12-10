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

/**
 * Durable A2A client wrapper that makes all A2A operations replayable via Restate.
 * All methods are wrapped with ctx.run() for durability.
 */
export class A2AClient {
	private ctx: ObjectContext;
	private client: BaseA2AClient;
	private agentId: string;

	constructor(ctx: ObjectContext, client: BaseA2AClient, agentId: string) {
		this.ctx = ctx;
		this.client = client;
		this.agentId = agentId;
	}

	/**
	 * Send a message to the agent (non-streaming, durable).
	 * The entire request/response cycle is wrapped for durability.
	 */
	async sendMessage(params: MessageSendParams): Promise<SendMessageResponse> {
		return this.ctx.run(`a2a-${this.agentId}-sendMessage`, async () => {
			return this.client.sendMessage(params);
		});
	}

	/**
	 * Send a message and stream responses.
	 * Each event from the stream is individually made durable.
	 * Returns an async generator that yields events durably.
	 */
	async *sendMessageStream(
		params: MessageSendParams,
	): AsyncGenerator<A2AStreamEventData, void, undefined> {
		// Start the stream (this creates the SSE connection)
		const stream = this.client.sendMessageStream(params);
		let eventIndex = 0;

		for await (const event of stream) {
			// Make each event durable by wrapping it in ctx.run
			const durableEvent = await this.ctx.run(
				`a2a-${this.agentId}-stream-event-${eventIndex}`,
				async () => event,
			);
			eventIndex++;
			yield durableEvent;
		}
	}

	/**
	 * Get a task by ID (durable).
	 */
	async getTask(params: TaskQueryParams): Promise<GetTaskResponse> {
		return this.ctx.run(
			`a2a-${this.agentId}-getTask-${params.id}`,
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
			`a2a-${this.agentId}-cancelTask-${params.id}`,
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
		let eventIndex = 0;

		for await (const event of stream) {
			const durableEvent = await this.ctx.run(
				`a2a-${this.agentId}-resubscribe-event-${params.id}-${eventIndex}`,
				async () => event,
			);
			eventIndex++;
			yield durableEvent;
		}
	}

	/**
	 * Get the agent card (durable).
	 */
	async getAgentCard(): Promise<AgentCard> {
		return this.ctx.run(`a2a-${this.agentId}-getAgentCard`, async () => {
			return this.client.getAgentCard();
		});
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
