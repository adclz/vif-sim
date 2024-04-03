import init, {boot_container, is_running, Container} from "../../plcsim.js";
import {ToCommand, From, Send} from "./types.js";

let container: Container
self.addEventListener("message", async (e: MessageEvent<ToCommand>) => {
    switch (e.data.command) {
        case Send.Boot:
            await checkServer(e.data.pause_sab, e.data.command_lock_sab)
            break;
        case Send.LoadContainerParams:
            if (!is_running())
                container.load_server_params(JSON.stringify(e.data.params))
            break;
        case Send.LoadPlugin:
            if (!is_running())
                self.postMessage({
                    command: From.LoadPlugin,
                    status: container.add_plugin(e.data.name, e.data.interval)
                })
            break;
        case Send.LoadProvider:
            if (!is_running())
                self.postMessage({
                    command: From.LoadProviderStatus,
                    status: container.load_provider(JSON.stringify(e.data.pack))
                })
            break;
        case Send.LoadProgram:
            if (!is_running()) {
                self.postMessage({
                    command: From.LoadProgramStatus,
                    sab: container.load_program(JSON.stringify(e.data.program))
                })
                self.postMessage({
                    command: From.RuntimeCommandsInt32,
                    sab: container.get_runtime_commands_int32()
                })
            }
            break;
        case Send.Start:
            if (!is_running())
                await container.start(e.data.main)
            break;
        case Send.ClearProgram:
            if (!is_running())
                container.clear_program()
            break;
        case Send.ClearProvider:
            if (!is_running())
                container.clear_provider()
            break;
    }
})
const checkServer = async (pause_sab: SharedArrayBuffer, command_lock_sab: SharedArrayBuffer) => {
    if (!container) {
        await init()
        container = boot_container(undefined, pause_sab, command_lock_sab)
        self.postMessage({command: From.ContainerReady, id: container.get_id()})
        setInterval(() => {
            if (!is_running())
                container.read_sab_commands();
        }, 100)
    }
}

export {}