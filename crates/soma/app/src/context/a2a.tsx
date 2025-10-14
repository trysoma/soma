// ---- types ----
import type { AgentCard, Message, Task, TaskStatus } from '@a2a-js/sdk'
import { createContext, Suspense, useCallback, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import { v4 } from 'uuid';
import type { ToolUIPart } from 'ai';
import { A2AClient } from '@a2a-js/sdk/client';
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
}

type SelectedIds = {
    contextId: string | null;
    taskId: string | null;
}

type A2aContextValue = {
    agentCard: AgentCard | null;
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
    const a2aTaskStream = useRef<{ generator: ReturnType<A2AClient['resubscribeTask']>, abort: AbortController } | null>(null);
    // manage the task stream
    const unsubscribeTaskStream = async () => {
        if (a2aTaskStream.current) {
            await a2aTaskStream.current.generator.return();
            a2aTaskStream.current.abort.abort();
            a2aTaskStream.current = null;
        }
    }

    const processTaskStream = async (generator: ReturnType<A2AClient['resubscribeTask']>) => {
        const abort = new AbortController();
        a2aTaskStream.current = {
            generator,
            abort
        };

        for await (const event of generator) {
            console.log('event', event);
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
                case 'task':{

                    setContexts((prev) => {
                        if (!prev) {
                            return prev;
                        }
                        const prevCopy = [...prev];
                        const cachedContext = prevCopy.find((c) => c.id === event.contextId);
                        if (!cachedContext) {
                            return prevCopy;
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
                    setSelectedIds((prev) => ({ ...prev, taskId: event.id    }));

                    break;
                }
                case 'artifact-update': {

                    break;
                }
                case 'status-update': {
                    console.log('status-update', event);
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
                    setSelectedIds((prev) => ({ ...prev, taskId: event.taskId ?? null    }));

                    if (event.final) {
                        await unsubscribeTaskStream();
                    }
                    break;
                }
                case 'message': {
                    setContexts((prev) => {
                        if (!prev) {
                            return prev;
                        }
                        const prevCopy = [...prev];
                        const cachedContext = prevCopy.find((c) => c.id === event.contextId);
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
                    setSelectedIds((prev) => ({ ...prev, taskId: event.taskId ?? null    }));

                    break;
                }
                    
            }

        }
    }

    const subscribeTaskStream = (taskId: string | null) => {
        if (!taskId || !a2aClient.current) {
            return;
        }

        const client = a2aClient.current;

        void (async () => {
            try {
                await unsubscribeTaskStream();
            }
            catch (error) {
                console.error('Failed to unsubscribe from previous task stream', error);
            }

            if (a2aTaskStream.current) {
                return;
            }

            if (a2aClient.current !== client) {
                return;
            }

            try {
                console.log('invoking processTaskStream');
                await processTaskStream(client.resubscribeTask({ id: taskId }));
            }
            catch (error) {
                console.error('Failed to subscribe to task stream', error);
            }
        })();
    };

    // Resolve selectedOrg on mount / when data changes
    useEffect(() => {
        (async () => {
            try {
                const agentCard = await fetch("/api/agent/v1/.well-known/agent.json");
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

    const createContext = useCallback((contextId: string) => {
        setContexts((prev) => ([...prev, { createdAt: new Date(), id: contextId, tasks: [], currentTask: null } as Context]));
        setSelectedIds((prev) => ({ ...prev, contextId: contextId }));
    }, []);

    const setCurrentContext = (contextId: string | null) => {
        if (contextId === null) {
            setSelectedIds({ contextId: null, taskId: null });
            return;
        }
        const context = contexts.find((context) => context.id === contextId);
        if (context) {
            setSelectedIds({ contextId: context.id, taskId: null });
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
            setSelectedIds((prev) => ({ ...prev, taskId: null }));
            return;
        }
        else {
            const context = contexts.find((context) => context.id === selectedIds.contextId);
            if (!context) {
                // TODO: handle this error case
                return;
            }


            if (!a2aClient.current) {
                // TODO: handle this error case
                return;
            }

            a2aTaskStream.current = {
                generator: a2aClient.current.resubscribeTask({
                    id: taskId,
                }),
                abort: new AbortController()
            };
            const res = await a2aClient.current.getTask({ id: taskId });

            if ('error' in res) {
                // TODO: handle this error case
                return;
            }

            const task = res.result;

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

                const constructTask = () => ({
                    somaView: {
                        // need to stuff soma data into a2a task metadata
                        createdAt: new Date(),
                        id: taskId
                    },
                    a2aView: task,
                    aiSdkView: { messages: task.history?.map((message) => mapA2aMessageToAiSdkMessage(message)) ?? [] }
                })

                if (cachedTaskIndex === -1) {
                    cachedContext.tasks.push(constructTask());

                }
                else {
                    cachedContext.tasks[cachedTaskIndex] = constructTask();
                }

                return prevCopy;
            });
            setSelectedIds((prev) => ({ ...prev, taskId: taskId }));
        }
    };

    const sendMessage = async (message: string) => {
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

        if (selectedIds.contextId === null) {
            console.error('No context ID found, cannot send message');
            // TODO: handle this error case
            return;
        }

        let resGenerator = a2aClient.current.sendMessageStream({
            message: {
                contextId: selectedIds.contextId,
                taskId: selectedIds.taskId ?? undefined,
                kind: 'message',
                messageId: v4(),
                parts: [{
                    kind: 'text',
                    text: message
                }],
                role: 'agent'
            },
        });
        console.log('invoking processTaskStream');
        processTaskStream(resGenerator);
    }

    const currentContext = contexts.find((context) => context.id === selectedIds.contextId) ?? null;
    const currentTask = currentContext?.tasks.find((task) => task.somaView.id === selectedIds.taskId) ?? null;


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
