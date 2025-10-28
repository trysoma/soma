import { Task } from "@a2a-js/sdk";
import * as restate from "@restatedev/restate-sdk";
import { RestatePromise } from "@restatedev/restate-sdk";
import {
  Configuration,
  DefaultApi,
  DefaultConfig,
  TaskTimelineItem
} from "@soma/api-client";
import z from "zod";

const DEFAULT_BASE_URL = "http://localhost:3000";

type HandlerParams = { task: Task; timelineItem: TaskTimelineItem };

interface HandlerCtx {
  ctx: {
    restate: restate.ObjectContext;
    soma: DefaultApi;
  };
  params: HandlerParams;
}
const constructCtx = (
  ctx: restate.ObjectContext,
  params: HandlerParams,
): HandlerCtx => {
  const api = new DefaultApi(new Configuration({
    basePath: DEFAULT_BASE_URL,
  }));
  return {
    ctx: {
      restate: ctx,
      soma: api,
    },
    params,
  };
};

const constructCancelId = (taskId: string) => `soma:v1:task:${taskId}:cancel`;
const constructInvocationKey = (taskId: string) => `soma:v1:task:${taskId}:invocation`;
const constructTaskKey = (taskId: string) => `soma:v1:task:${taskId}`;
type CancelId = string;
type CancelResult = "cancelled";
type ProcessMessageHandler = (
  ctx: restate.ObjectContext,
  params: HandlerParams,
) => Promise<void>;

interface SomaFunction<InputType, OutputType> {
  inputSchema: z.ZodSchema<InputType>;
  outputSchema: z.ZodSchema<OutputType>;
  handler: (input: InputType) => Promise<OutputType>;
}

interface CreateSomaAgentParams {
  handler: (ctx: HandlerCtx) => Promise<void>;
}

export function createSomaAgent(
  agentParams: CreateSomaAgentParams,
) {
  // --- shared API client config
  // const somaApi = new DefaultApi(new Configuration({
  //   basePath: DEFAULT_BASE_URL,
  // }));
  // const somaConfig = await somaApi.getAgentDefinition();

  // --- single service definition
  // const service = restate.object({

  //   name: somaConfig.project,

  //   handlers: {
  //     onNewMessage: async (
  //       ctx: restate.ObjectContext,
  //       params: HandlerParams,
  //     ) => {
  //       const { task } = params;
  //       const cancelKey = constructCancelId(task.id);

  //       // cancel any previous in-flight invocation
  //       const prevCancelId = await ctx.get<CancelId>(cancelKey);
  //       if (prevCancelId) {
  //         ctx.console.info(`Cancelling previous run for task ${task.id}`);
  //         ctx.resolveAwakeable(prevCancelId, "cancelled");
  //       }

  //       // create new durable cancel token
  //       const { id: cancelId, promise: cancelPromise } =
  //         ctx.awakeable<CancelResult>();
  //       await ctx.set(cancelKey, cancelId);
  //       ctx.console.info(`Set cancel id for task ${task.id}: ${cancelId}`);

  //       const controller = new AbortController();

  //       try {
  //         const invocation = ctx.objectClient<{ handler: ProcessMessageHandler }>({
  //           name: somaConfig.project,
  //         }, constructInvocationKey(task.id))
  //           .handler(params);

  //         // race the user handler and cancellation signal
  //         const result = await RestatePromise.race([
  //           invocation,
  //           cancelPromise,
  //         ]);

  //         if (result === "cancelled") {
  //           ctx.console.info(`Handler cancelled for ${task.id}`);
  //           controller.abort();
  //         }
  //         else {
  //           ctx.console.info(`Handler completed for ${task.id}`);

  //         }

  //       } catch (err) {
  //         ctx.console.error(
  //           `Handler failed for ${task.id}: ${(err as Error).stack}`,
  //         );
  //       } finally {
  //         // resolve durable promise to prevent leaks
  //         await ctx.resolveAwakeable(cancelId, "complete");
  //         await ctx.set(cancelKey, undefined);
  //       }
  //     },
  //     handler: async (ctx: restate.ObjectContext, params: HandlerParams) => agentParams.handler(constructCtx(ctx, params))
  //   },
  // });

  // return restate.serve({ services: [service] });
}

export * from '@soma/sdk-core'
