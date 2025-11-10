// import { Task } from "@a2a-js/sdk";
// import * as restate from "@restatedev/restate-sdk";
// import { RestatePromise } from "@restatedev/restate-sdk";
// import {
//   Configuration,
//   DefaultApi,
//   DefaultConfig,
//   TaskTimelineItem
// } from "@soma/api-client";
// import z from "zod";

// const DEFAULT_BASE_URL = "http://localhost:3000";

// type HandlerParams = { task: Task; timelineItem: TaskTimelineItem };

// interface HandlerCtx {
//   ctx: {
//     restate: restate.ObjectContext;
//     soma: DefaultApi;
//   };
//   params: HandlerParams;
// }
// const constructCtx = (
//   ctx: restate.ObjectContext,
//   params: HandlerParams,
// ): HandlerCtx => {
//   const api = new DefaultApi(new Configuration({
//     basePath: DEFAULT_BASE_URL,
//   }));
//   return {
//     ctx: {
//       restate: ctx,
//       soma: api,
//     },
//     params,
//   };
// };

// const constructCancelId = (taskId: string) => `soma:v1:task:${taskId}:cancel`;
// const constructInvocationKey = (taskId: string) => `soma:v1:task:${taskId}:invocation`;
// const constructTaskKey = (taskId: string) => `soma:v1:task:${taskId}`;
// type CancelId = string;
// type CancelResult = "cancelled";
// type ProcessMessageHandler = (
//   ctx: restate.ObjectContext,
//   params: HandlerParams,
// ) => Promise<void>;

// interface SomaFunction<InputType, OutputType> {
//   inputSchema: z.ZodSchema<InputType>;
//   outputSchema: z.ZodSchema<OutputType>;
//   handler: (input: InputType) => Promise<OutputType>;
// }

// interface CreateSomaAgentParams {
//   handler: (ctx: HandlerCtx) => Promise<void>;
// }

// // export const checkpoint = async (ctx: restate.ObjectContext, stage: ()=> Promise<void>) => {
// //   const { id, promise } = ctx.awakeable<string>();

// //   await RestatePromise.race([
// //     promise,
// //     RestatePromise.from(stage()),
// //   ])
// // }

// export function createSomaAgent(
//   agentParams: CreateSomaAgentParams,
// ) {

//   const service = restate.object({
//     name: "soma-agent",
//     handlers: {
//       // agent: async (ctx: restate.ObjectContext, params: HandlerParams) => {
//       //   let state = (await ctx.get()) ?? { messages: params.messages ?? [] };
        
//       //   await sendEmail(ctx, state);
//       //   await ctx.maybe_await_signal("new_message");  // interrupt
  
//       //   await processRefund(ctx, state);
//       //   await ctx.maybe_await_signal("new_message");  // interrupt
  
//       //   while (true) {
//       //     state = await ctx.get();
//       //     const llmMessage = await llm.generate(ctx, state.messages);
//       //     await sendMessage(ctx, state, llmMessage);
//       //     const newMsg = await ctx.await_signal("new_message");
//       //     state.messages.push(newMsg);
//       //     await ctx.set(state);
//       //   }
//       // },
  
//       // add_message: async (ctx, message) => {
//       //   const state = (await ctx.get()) ?? { messages: [] };
//       //   state.messages.push(message);
//       //   await ctx.set(state);
//       //   ctx.signal("new_message", message);
//       // },
//     },
//   });

// }
export * from '@soma/sdk-core'
export * from './agent'
export * from './bridge'
export * from './patterns'
export { patterns } from './patterns'
