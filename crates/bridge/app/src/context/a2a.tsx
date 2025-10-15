// ---- types ----
import type { AgentCard, Message, Part, Task, TaskStatus } from '@a2a-js/sdk'
import { createContext, Suspense, useCallback, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import { v4 } from 'uuid';
import type { ToolUIPart } from 'ai';
import { A2AClient } from '@a2a-js/sdk/client';
import $api from '@/lib/api-client';
import type { components } from '@/@types/openapi';
export const DEFAULT_AGENT_CARD_PATH = "/api/a2a/v1/.well-known/agent.json";
export const DEFAULT_AGENT_SSE_PATH = "/api/a2a/v1"
// type User = Awaited<WhoAmI>;

// type UserContextValue = {
//   user: User | null;
//   refetchUser: () => void;
//   selectedOrg: SelectedOrganisation ;
//   setSelectedOrg: (org_id: string ) => void;
// };

export type MessageType = {
    key: string;
    from: 'user' | 'assistant';
    sources?: { href: string; title: string }[];
    versions: {
        id: string;
        content: string;
    }[];
    reasoning?: {
        content: string;
        duration: number;
    };
    tools?: {
        type: ToolUIPart["type"];
        description: string;
        state: ToolUIPart['state'];
        parameters: Record<string, unknown>;
        result: string | undefined;
        error: string | undefined;
    }[];
    avatar: string;
    name: string;
};
interface WrappedTask {
    somaView: {
        createdAt: Date;
        id: string;
    }
    a2aView: Task;
    aiSdkView: {
        messages: MessageType[];
    };
}
type Context = {
    createdAt: Date;
    id: string;
    tasks: WrappedTask[];
    taskNextToken: string | null;
}

type SelectedIds = {
    contextId: string | null;
    taskId: string | null;
}

type A2aContextValue = {
    agentCard: AgentCard;
    contexts: Context[];
    selectedIds: SelectedIds;
    setCurrentContext: (contextId: string | null) => void;
    createContext: (contextId: string) => void;
    setCurrentTask: (taskId: string | null) => void;
    sendMessage: (message: string) => void;
    a2aClient: A2AClient | null;
    isReady: boolean;
    currentContext: Context | null;
    currentTask: WrappedTask | null;
}

const A2aContext = createContext<A2aContextValue | undefined>(undefined);

export function useA2a() {
    const ctx = useContext(A2aContext);
    if (!ctx) {
        throw new Error("useA2a must be used within <A2aProvider>");
    }
    return ctx;
}


const mapA2aMessageToAiSdkMessage = (message: Message): MessageType => {
    return {
        key: message.messageId,
        from: message.role === 'user' ? 'user' : 'assistant',
        versions: [{
            id: message.messageId, content: message.parts.map((part) => {
                if (part.kind === 'text') {
                    return part.text;
                }
                return '';
            }).join('')
        }],
        avatar: message.role === 'user' ? 'user' : 'assistant',
        name: message.role === 'user' ? 'User' : 'Assistant',
        tools: []
    };
}

const mapA2aTaskToWrappedTask = (task: Task): WrappedTask => {
    return {
        somaView: {
            createdAt: new Date(),
            id: task.id
        },
        a2aView: task,
        aiSdkView: { messages: task.history?.map((message) => mapA2aMessageToAiSdkMessage(message)) ?? [] }
    }
}

const mapSomaPartToA2aPart = (part: components['schemas']['MessagePart']): Part => {
    switch (part.type) {
        case 'text-part':
            return {
                kind: 'text',
                text: part.text
            }
    }
}

const mapSomaMessageToA2aMessage = (message: components['schemas']['Message'], contextId?: string): Message => {
    return {
        contextId: contextId,
        taskId: message.task_id,
        metadata: message.metadata,
        referenceTaskIds: message.reference_task_ids,
        kind: 'message',
        messageId: message.id,
        role: message.role,
        parts: message.parts.map((part) => mapSomaPartToA2aPart(part))
    }
}

const mapSomaMessageToAiSdkMessage = (message: components['schemas']['Message']): MessageType => {
    return {
        key: message.id,
        from: message.role === 'user' ? 'user' : 'assistant',
        versions: [{
            id: message.id, content: message.parts.map((part) => {
                if (part.type === 'text-part') {
                    return part.text;
                }
                return '';
            }).join('')
        }],
        avatar: message.role === 'user' ? 'user' : 'assistant',
        name: message.role === 'user' ? 'User' : 'Assistant',
        tools: []
    }
}
const mapSomaTaskToWrappedTask = (task: components['schemas']['TaskWithDetails']['task'], status_message?: components['schemas']['TaskWithDetails']['status_message'], messages?: components['schemas']['TaskWithDetails']['messages']): WrappedTask => {
    return {
        somaView: {
            createdAt: new Date(task.created_at),
            id: task.id
        },
        a2aView: {
            contextId: task.context_id,
            kind: 'task',
            id: task.id,
            status: {
                state: task.status,
                // message: task.status_message_id,
                message: status_message ? mapSomaMessageToA2aMessage(status_message) : undefined,
                timestamp: task.status_timestamp
            },
        },
        aiSdkView: { messages: messages?.map((message) => mapSomaMessageToAiSdkMessage(message)) ?? [] }
    }
}

const mapA2aTaskStatusUpdateToWrappedTask = (wrappedTask: WrappedTask, taskStatusUpdate: TaskStatus): WrappedTask => {
    wrappedTask.a2aView.status = taskStatusUpdate;

    // Check if a status update message for this status already exists
    const statusUpdateExists = wrappedTask.aiSdkView.messages.some((msg) =>
        msg.tools?.some((tool) =>
            tool.type === 'tool-task-status-update' &&
            tool.parameters.status === taskStatusUpdate.state
        )
    );

    if (statusUpdateExists) {
        return wrappedTask;
    }

    const messageId = v4();

    // Extract message content from status update if available
    const statusMessage = taskStatusUpdate.message;
    let messageContent = '';

    if (statusMessage && statusMessage.parts) {
        messageContent = statusMessage.parts.map((part) => {
            if (part.kind === 'text') {
                return part.text;
            }
            return '';
        }).join('');
    }

    wrappedTask.aiSdkView.messages.push({
        key: messageId, // Generate unique key for each status update
        from: 'assistant',
        versions: messageContent ? [{
            id: messageId,
            content: messageContent
        }] : [],
        avatar: 'assistant',
        name: 'Assistant',
        tools: [
            {
                type: 'tool-task-status-update',
                description: 'Update the status of the task',
                state: 'output-available',
                parameters: {
                    taskId: wrappedTask.somaView.id,
                    status: taskStatusUpdate.state,
                },
                result: `Task status updated to ${taskStatusUpdate.state}`,
                error: undefined,
            }
        ]
    });
    return wrappedTask;
}
function A2aProviderInner({ children }: { children: ReactNode }) {
    const [ready, setReady] = useState(false);
    const [agentCard, setAgentCard] = useState<AgentCard | null>(null);
    const [contexts, setContexts] = useState<Context[]>([]);
    const [selectedIds, setSelectedIds] = useState<{ contextId: string | null, taskId: string | null }>({ contextId: null, taskId: null });
    const a2aClient = useRef<A2AClient | null>(null);
    const a2aTaskStream = useRef<{ generator: ReturnType<A2AClient['resubscribeTask']>, abort: AbortController, taskId: string | null } | null>(null);
    const [curContextPageToken, setCurContextPageToken] = useState<string | null>(null);
    const fetchPageOfContexts = async (pageToken: string | null) => {
        const res = await $api.GET('/api/task/v1/context', {
            params: {
                query: {
                    page_size: 1000,
                    page_token: pageToken
                }
            }
        });
        if ('error' in res) {
            console.error('Failed to fetch page of contexts', res.error);
            return [];
        }

        setCurContextPageToken(res.data.next_page_token);
        setContexts((prev) => {
            if (!prev) {
                return prev;
            }
            const prevCopy = [...prev];
            const newContexts = res.data.items;

            for (const newContext of newContexts) {
                const foundContext = prevCopy.find((c) => c.id === newContext.context_id);
                if (foundContext == null) {
                    prevCopy.push({
                        createdAt: new Date(newContext.created_at),
                        id: newContext.context_id,
                        tasks: [],
                        taskNextToken: null
                    });
                }
            }

            return prevCopy;
        });


    }

    const fetchPageOfTasks = async (contextId: string, pageToken: string | null) => {
        console.log('fetchPageOfTasks', contextId, pageToken);
        const res = await $api.GET('/api/task/v1/context/{context_id}/task', {
            params: {
                path: {
                    context_id: contextId
                },
                query: {
                    page_size: 1000,
                }
            }
        });
        if ('error' in res) {
            console.error('Failed to fetch page of tasks', res.error);
            return [];
        }
        setContexts((prev) => {
            if (!prev) {
                return prev;
            }
            const prevCopy = [...prev];
            const cachedContext = prevCopy.find((c) => c.id === contextId);
            if (!cachedContext) {
                return prevCopy;
            }

            for (const remoteTask of res.data.items) {
                const foundTask = cachedContext.tasks.find((t) => t.somaView.id === remoteTask.id);
                if (foundTask == null) {
                    cachedContext.tasks.push(mapSomaTaskToWrappedTask(remoteTask, undefined, undefined));
                }
            }
            cachedContext.taskNextToken = res.data.next_page_token;
            return prevCopy;
        });
    }

    useEffect(() => {
        fetchPageOfContexts(null);
    }, [])

    // manage the task stream
    const unsubscribeTaskStream = async () => {
        if (a2aTaskStream.current) {
            // TODO: a2ajs does not support aborting the SSE stream
            // await a2aTaskStream.current.generator.return();
            a2aTaskStream.current.abort.abort();
            a2aTaskStream.current = null;
        }
    }

    const processTaskStream = async (generator: ReturnType<A2AClient['resubscribeTask']>, taskId: string | null) => {
        // Clean up any existing stream before starting a new one
        await unsubscribeTaskStream();
        // TODO: a2ajs does not support aborting the SSE stream
        const abort = new AbortController();
        a2aTaskStream.current = {
            generator,
            abort,
            taskId: taskId
        };

        for await (const event of generator) {
            // console.log('event', event);
            // if(event.type === 'task') {
            //   setContexts((prev) => {
            //     if(!prev) {
            //       return prev;
            //     }
            //     return prev;
            //   });
            // }
            switch (event.kind) {
                // TODO: the A2A async generator stream has a race condition where the task is created and then the status is updated in seperate messages however 
                // internally the task is updated in the a2a client before the first message is sent. meaning the initial task already is updated to status "input-required", etc.
                // instead of submitted
                case 'task': {
                    // taskId can be null if it's the first message
                    if (a2aTaskStream.current?.taskId !== event.id) {
                        a2aTaskStream.current.taskId = event.id;
                        console.log('taskId for stream changed to', event.id);
                    }
                    setContexts((prev) => {
                        if (!prev) {
                            return prev;
                        }
                        const prevCopy = [...prev];
                        let cachedContext = prevCopy.find((c) => c.id === event.contextId);
                        // if (!cachedContext) {
                        //     return prevCopy;
                        // }
                        if (!cachedContext) {
                            cachedContext = {
                                createdAt: new Date(),
                                id: event.contextId,
                                tasks: [],
                                taskNextToken: null
                            };
                            prevCopy.push(cachedContext);
                            setSelectedIds((prev) => ({ ...prev, contextId: event.contextId }));
                        }
                        const cachedTask = cachedContext.tasks.find((t) => t.somaView.id === event.id);
                        // Only add task if it doesn't already exist
                        if (!cachedTask) {
                            const newTask: WrappedTask = mapA2aTaskToWrappedTask(event);
                            newTask.aiSdkView.messages.push({
                                key: v4(),
                                from: 'assistant',
                                versions: [],
                                avatar: 'assistant',
                                name: 'Assistant',
                                tools: [
                                    {
                                        type: 'tool-task-status-update',
                                        description: 'Update the status of the task',
                                        state: 'output-available',
                                        parameters: {
                                            taskId: event.id,
                                            status: 'submitted',
                                        },
                                        result: 'Task created',
                                        error: undefined,
                                    }
                                ],
                            });
                            cachedContext.tasks.push(newTask);
                        }

                        return prevCopy;
                    });
                    setSelectedIds((prev) => ({ ...prev, taskId: event.id }));

                    break;
                }
                case 'artifact-update': {

                    break;
                }
                case 'status-update': {
                    setContexts((prev) => {
                        if (!prev) {
                            return prev;
                        }
                        const prevCopy = [...prev];
                        const cachedContext = prevCopy.find((c) => c.id === event.contextId);
                        if (!cachedContext) {
                            return prevCopy;
                        }
                        let cachedTask = cachedContext.tasks.find((t) => t.somaView.id === event.taskId);
                        if (!cachedTask) {
                            return prevCopy;
                        }
                        cachedTask = mapA2aTaskStatusUpdateToWrappedTask(cachedTask, event.status);

                        return prevCopy;
                    });
                    setSelectedIds((prev) => ({ ...prev, taskId: event.taskId ?? null }));

                    if (event.final) {
                        await unsubscribeTaskStream();
                    }
                    break;
                }
                case 'message': {
                    setContexts((prev) => {
                        console.log('message', event, prev);
                        if (!prev) {
                            return prev;
                        }
                        const prevCopy = [...prev];

                        // Find context by contextId if available, otherwise search by taskId
                        let cachedContext = event.contextId
                            ? prevCopy.find((c) => c.id === event.contextId)
                            : prevCopy.find((c) => c.tasks.some((t) => t.somaView.id === event.taskId));

                        if (!cachedContext) {
                            return prevCopy;
                        }
                        const cachedTask = cachedContext.tasks.find((t) => t.somaView.id === event.taskId);
                        if (!cachedTask) {
                            return prevCopy;
                        }
                        if (cachedTask.a2aView.history == null) {
                            cachedTask.a2aView.history = [];
                        }

                        // Check if message already exists in history
                        const messageExists = cachedTask.a2aView.history.some((msg) => msg.messageId === event.messageId);
                        if (messageExists) {
                            return prevCopy;
                        }

                        // Check if message already exists in aiSdkView
                        const aiMessageExists = cachedTask.aiSdkView.messages.some((msg) => msg.key === event.messageId);
                        if (aiMessageExists) {
                            return prevCopy;
                        }

                        cachedTask.a2aView.history.push(event);
                        cachedTask.aiSdkView.messages.push({
                            key: event.messageId,
                            from: event.role === 'user' ? 'user' : 'assistant',
                            versions: [{
                                id: event.messageId,
                                content: event.parts.map((part) => {
                                    // TODO: handle other parts
                                    if (part.kind === 'text') {
                                        return part.text;
                                    }
                                    return '';
                                }).join(''),
                            }],
                            avatar: event.role === 'user' ? 'user' : 'assistant',
                            name: event.role === 'user' ? 'User' : 'Assistant',
                        });
                        // TODO: update soma view
                        return prevCopy;
                    });
                    setSelectedIds((prev) => ({ ...prev, taskId: event.taskId ?? null }));

                    break;
                }

            }

        }
    }

    // Resolve selectedOrg on mount / when data changes
    useEffect(() => {
        (async () => {
            try {
                const agentCard = await fetch(DEFAULT_AGENT_CARD_PATH);
                if (!agentCard.ok) {
                    throw new Error(`Failed to fetch agent card: ${agentCard.status}`);
                }
                const agentCardJson: AgentCard = await agentCard.json();
                a2aClient.current = new A2AClient(agentCardJson);

                setAgentCard(agentCardJson);
                setReady(true);
            } catch (error) {
                console.error("Error initializing A2A client:", error);
            }
        })();
    }, []);

    // Cleanup on unmount
    useEffect(() => {
        return () => {
            void unsubscribeTaskStream();
        };
    }, []);

    const createContext = useCallback((contextId: string) => {
        setContexts((prev) => ([...prev, { createdAt: new Date(), id: contextId, tasks: [], taskNextToken: null }]));
        setSelectedIds((prev) => ({ ...prev, contextId: contextId, taskId: null }));
    }, []);

    const setCurrentContext = async (contextId: string | null) => {
        await unsubscribeTaskStream();
        if (contextId === null) {
            setSelectedIds({ contextId: null, taskId: null });
            return;
        }
        const context = contexts.find((context) => context.id === contextId);
        if (context) {
            setSelectedIds({ contextId: context.id, taskId: null });
            if (context.tasks.length === 0) {
                fetchPageOfTasks(context.id, null);
            }
        }
    };

    const setCurrentTask = async (taskId: string | null) => {
        if (taskId === selectedIds.taskId) {
            if (a2aTaskStream.current) {
                // a2aMsgStream.current.
                // stop generator
            }
            return;
        }


        if (taskId === null) {
            await unsubscribeTaskStream();
            setSelectedIds((prev) => ({ ...prev, taskId: null }));
            return;
        }
        else {
            const context = contexts.find((context) => context.id === selectedIds.contextId);
            if (!context) {
                console.error('No context found, cannot set current task');
                // TODO: handle this error case
                return;
            }


            if (!a2aClient.current) {
                console.error("No A2A client found, cannot set current task");
                // TODO: handle this error case
                return;
            }

            // Fetch task details first
            let cachedTask = context.tasks.find((t) => t.somaView.id === taskId);
            
            if (!cachedTask || cachedTask.aiSdkView.messages.length === 0) {
                const res = await $api.GET('/api/task/v1/{task_id}', {
                    params: {
                        path: {
                            task_id: taskId
                        },
                    }
                });
                if ('error' in res) {
                    console.error('Failed to get task', res.error);
                    return;
                }
                const somaTask = res.data;
                cachedTask = mapSomaTaskToWrappedTask(somaTask.task, somaTask.status_message, somaTask.messages);
            }

            setContexts((prev) => {
                if (!prev) {
                    return prev;
                }
                const prevCopy = [...prev];
                const cachedContext = prevCopy.find((c) => c.id === context.id);
                if (!cachedContext) {
                    return prevCopy;
                }

                const cachedTaskIndex = cachedContext.tasks.findIndex((t) => t.somaView.id === taskId);

                if (cachedTaskIndex === -1) {
                    cachedContext.tasks.push(cachedTask);

                }
                else {
                    cachedContext.tasks[cachedTaskIndex] = cachedTask;
                }

                return prevCopy;
            });
            setSelectedIds((prev) => ({ ...prev, taskId: taskId }));

            // Subscribe to task stream to receive real-time updates
            // Don't await this - let it run in the background
            const generator = a2aClient.current.resubscribeTask({ id: taskId });
            processTaskStream(generator, taskId).catch((error) => {
                console.error('Failed to process task stream', error);
            });
        }
    };

    const sendMessage = async (message: string) => {
        console.log('sendMessage called with:', message, 'selectedIds:', selectedIds);
        if (!a2aClient.current) {
            console.error('No A2A client found, cannot send message');
            // TODO: handle this error case
            return;
        }
        // TASK ID can be null
        // if (selectedIds.taskId === null) {
        //   // TODO: handle this error case
        //   return;
        // }

        // if (selectedIds.contextId === null) {
        //     console.error('No context ID found, cannot send message');
        //     // TODO: handle this error case
        //     return;
        // }

        try {
            console.log('Current stream taskId:', a2aTaskStream.current?.taskId);
            console.log('Selected taskId:', selectedIds.taskId);

            // Use sendMessageStream if:
            // 1. No active stream exists, OR
            // 2. The taskId has changed (including null -> some taskId or some taskId -> null)
            const needsNewStream = !a2aTaskStream.current || a2aTaskStream.current.taskId !== selectedIds.taskId;

            if (needsNewStream) {
                console.log('Creating new stream with sendMessageStream...');

                let resGenerator = a2aClient.current.sendMessageStream({
                    message: {
                        contextId: selectedIds.contextId ?? undefined,
                        taskId: selectedIds.taskId ?? undefined,
                        kind: 'message',
                        messageId: v4(),
                        parts: [{
                            kind: 'text',
                            text: message
                        }],
                        role: 'user'
                    },
                });
                console.log('Processing task stream...');
                await processTaskStream(resGenerator, selectedIds.taskId);
                console.log('Task stream processing complete');
            }
            else {
                console.log('Using existing stream with sendMessage...');
                await a2aClient.current.sendMessage({
                    message: {
                        contextId: selectedIds.contextId ?? undefined,
                        taskId: selectedIds.taskId ?? undefined,
                        kind: 'message',
                        messageId: v4(),
                        parts: [{
                            kind: 'text',
                            text: message
                        }],
                        role: 'user'
                    },
                });
            }

        } catch (error) {
            console.error('Failed to send message', error);
        }
    }

    const currentContext = contexts.find((context) => context.id === selectedIds.contextId) ?? null;
    const currentTask = currentContext?.tasks.find((task) => task.somaView.id === selectedIds.taskId) ?? null;


    if (!agentCard || !a2aClient.current || !ready) {
        return <div className="flex items-center justify-center h-screen">Loading...</div>;
    }
    return (
        <A2aContext.Provider
            value={{
                agentCard,
                contexts,
                createContext,
                selectedIds,
                setCurrentContext,
                setCurrentTask,
                sendMessage,
                currentContext,
                currentTask,
                a2aClient: a2aClient.current,
                isReady: ready
            }}
        >
            {ready && agentCard && a2aClient.current ? children : <div className="flex items-center justify-center h-screen">Loading...</div>}
        </A2aContext.Provider>
    );
}

export function A2aProvider({ children }: { children: ReactNode }) {
    return (
        <Suspense fallback={<></>}>
            <A2aProviderInner>{children}</A2aProviderInner>
        </Suspense>
    );
}
