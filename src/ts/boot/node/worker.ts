import * as PlcSim from "../../plcsim.js"
import {ToCommand, From, Send} from "./types.js";
import {parentPort} from "node:worker_threads"

let container: PlcSim.Container
parentPort.on("message", async (e: ToCommand) => {
    switch (e.command) {
        case Send.Boot:
            checkServer(e.pause_sab, e.command_lock_sab)
            break;
        case Send.LoadContainerParams:
            if (!PlcSim.is_running())
                container.load_server_params(JSON.stringify(e.params))
            break;
        case Send.LoadPlugin:
            if (!PlcSim.is_running())
                parentPort.postMessage({
                    command: From.LoadPlugin,
                    status: container.add_plugin(e.name, e.interval)
                })
            break;
        case Send.LoadProvider:
            if (!PlcSim.is_running())
                parentPort.postMessage({
                    command: From.LoadProviderStatus,
                    status: container.load_provider(JSON.stringify(e.pack))
                })
            break;
        case Send.LoadProgram:
            if (!PlcSim.is_running()) {
                parentPort.postMessage({
                    command: From.LoadProgramStatus,
                    status: container.load_program(JSON.stringify(e.program))
                })
                parentPort.postMessage({
                    command: From.RuntimeCommandsInt32,
                    sab: container.get_runtime_commands_int32()
                })
            }
            break;
        case Send.Start:
            if (!PlcSim.is_running())
                await container.start(e.main)
            break;
        case Send.ClearProgram:
            if (!PlcSim.is_running())
                container.clear_program()
            break;
        case Send.ClearProvider:
            if (!PlcSim.is_running())
                container.clear_provider()
            break;
    }
})
const checkServer = (pause_sab: SharedArrayBuffer, command_lock_sab: SharedArrayBuffer) => {
    if (!container) {
        container = PlcSim.boot_container(undefined, pause_sab, command_lock_sab)
        parentPort.postMessage({command: From.ContainerReady, id: container.get_id()})
        setInterval(() => {
            if (!PlcSim.is_running())
                container.read_sab_commands();
        }, 100)
    }
}