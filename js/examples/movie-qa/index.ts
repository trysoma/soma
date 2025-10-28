import { createSomaAgent } from "@soma/sdk";
import { MessagePartTypeEnum, MessageRole, TaskStatus } from "../../packages/api-client/dist";

createSomaAgent({
	handler: async ({ ctx: { restate, soma }, params: { task, timelineItem } }) => {
		restate.console.info("Processing task update...", { text: timelineItem });
		// let taskHistory = await restate.run(async ()=> soma.taskHistory(10, task.id));
		// restate.console.info("Task history...", { text: taskHistory });
		// update the task status
		// await restate.run(()=> soma.updateTaskStatus(task.id, {
		// 	status: TaskStatus.InputRequired,
		// 	message: {
		// 		metadata: {},
		// 		parts: [{
		// 			text: "Processing task update...",
		// 			metadata: {},
		// 			type: MessagePartTypeEnum.TextPart,
					
		// 		}],
		// 		referenceTaskIds: [],
		// 		role: MessageRole.Agent,
		// 	}
		// }));
		
		while (true) {
			
			restate.console.info("Sleeping for 5 seconds...");
			await restate.sleep(5000);
			// throw new Error("Test error");
		}
	},
});


