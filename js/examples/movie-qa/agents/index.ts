import { createSomaAgent } from "@soma/sdk";
import {generatedBridgeClient} from './../.soma/bridge-client'

// const agent = await createSomaAgent<DefaultApi>({
// 	handler: async ({ ctx: { restate, soma }, params: { task, timelineItem } }) => {
// 		const client = new DefaultApi(new Configuration({
// 			basePath: 'http://localhost:3000',
// 		}));
// 		const res = await client.invokeNoneRandomNumber({
// 			randomNumberrandomNumberParams: {
// 				min: 1,
// 				max: 100,
// 			}
// 		})
// 		restate.console.info("Processing task update...", { text: timelineItem });
// 		// let taskHistory = await restate.run(async ()=> soma.taskHistory(10, task.id));
// 		// restate.console.info("Task history...", { text: taskHistory });
// 		// update the task status
// 		// await restate.run(()=> soma.updateTaskStatus(task.id, {
// 		// 	status: TaskStatus.InputRequired,
// 		// 	message: {
// 		// 		metadata: {},
// 		// 		parts: [{
// 		// 			text: "Processing task update...",
// 		// 			metadata: {},
// 		// 			type: MessagePartTypeEnum.TextPart,
					
// 		// 		}],
// 		// 		referenceTaskIds: [],
// 		// 		role: MessageRole.Agent,
// 		// 	}
// 		// }));
		
// 		while (true) {
			
// 			restate.console.info("Sleeping for 5 seconds...");
// 			await restate.sleep(5000);
// 			// throw new Error("Test error");
// 		}
// 	},
// });

// export default agent


export default createSomaAgent({
	generatedBridgeClient,
	projectId: "danielblignaut",
	agentId: "movieQA",
	name: "Movie Agent",
	description: "An agent that can answer questions about movies and actors using TMDB.",
	handlers: {
		agent: async ({ ctx, bridge }) => {
			ctx.console.info("Processing task update...");
			const res = await bridge.invokeNoneRandomNumber({
				randomNumberrandomNumberParams: {
					min: 1,
					max: 100,
				}
			});
			ctx.console.info("Random number...", { text: res });
		},
	},
});