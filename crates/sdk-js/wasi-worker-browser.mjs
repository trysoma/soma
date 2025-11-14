import {
	instantiateNapiModuleSync,
	MessageHandler,
	WASI,
} from "@napi-rs/wasm-runtime";

const handler = new MessageHandler({
	onLoad({ wasmModule, wasmMemory }) {
		const wasi = new WASI({
			print: () => {
				// biome-ignore lint/complexity/noArguments: NAPI default implementation
				console.log.apply(console, arguments);
			},
			printErr: () => {
				// biome-ignore lint/complexity/noArguments: NAPI default implementation
				console.error.apply(console, arguments);
			},
		});
		return instantiateNapiModuleSync(wasmModule, {
			childThread: true,
			wasi,
			overwriteImports(importObject) {
				importObject.env = {
					...importObject.env,
					...importObject.napi,
					...importObject.emnapi,
					memory: wasmMemory,
				};
			},
		});
	},
});

globalThis.onmessage = (e) => {
	handler.handle(e);
};
