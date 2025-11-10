// import { Artifact, Message, TaskStatus } from "@a2a-js/sdk";
import * as restate from "@restatedev/restate-sdk";
import { DefaultApi } from "@soma/api-client";
import { BaseAPI, Configuration as BridgeConfiguration } from './bridge';

export interface HandlerParams<T> {
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
    bridge: T
}

interface CreateSomaAgentParams<T> {
    generatedBridgeClient: (basePath: string) => T
    projectId: string
    agentId: string
    name: string
    description: string
    entrypoint: (params: HandlerParams<T>) => Promise<void>
}

export type SomaAgent<T> = CreateSomaAgentParams<T>

export const createSomaAgent = <T>(params: CreateSomaAgentParams<T>): SomaAgent<T> => {
    return {
        ...params,
    }
}