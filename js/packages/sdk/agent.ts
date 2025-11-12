// import { Artifact, Message, TaskStatus } from "@a2a-js/sdk";
import * as restate from "@restatedev/restate-sdk";
import { DefaultApi } from "@soma/api-client";

export interface HandlerParams {
    taskId: string;
    contextId: string;
    // TODO: add support for artifacts
    // artifacts: Artifact[]
    // TODO: add support for metadata
    // // metadata: Metadata
    // messages: Message[]
    // status: TaskStatus['state']
    ctx: restate.ObjectContext;
    soma: DefaultApi
}

interface CreateSomaAgentParams {
    projectId: string
    agentId: string
    name: string
    description: string
    entrypoint: (params: HandlerParams) => Promise<void>
}

export type SomaAgent = CreateSomaAgentParams

export const createSomaAgent = (params: CreateSomaAgentParams): SomaAgent => {
    return {
        ...params,
    }
}