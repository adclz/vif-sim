import {ParseStatus, ContainerParams} from "../../plcsim.js"
import {From, FromCommand, Send} from "./types.js";
import {CommandStore} from "./command-store.js";
import {VifEventEmitter} from "../event/event-emitter.js";

import path from "node:path"
import {Worker}  from "node:worker_threads"

type Hooks = {
    "container:ready": (id: string) => void
    "container:error": (error: ErrorEvent) => void
    "parse:provider": (status: ParseStatus) => void
    "parse:program": (status: ParseStatus) => void
    "plugin:loaded": (ev: { name: string | undefined, status: 0 | 1 }) => void
}

export class Container extends VifEventEmitter<Hooks> {
    worker: Worker
    pause_sab: SharedArrayBuffer
    pause_int32: Int32Array

    command_lock_sab: SharedArrayBuffer
    command_lock_int32: Int32Array

    command_store: CommandStore | null = null

    container_id: string | null = null

    constructor() {
        super()
        this.pause_sab =  new SharedArrayBuffer(8)
        this.pause_int32 = new Int32Array(this.pause_sab)
        Atomics.store(this.pause_int32, 0, 1)

        this.command_lock_sab =  new SharedArrayBuffer(8)
        this.command_lock_int32 = new Int32Array(this.command_lock_sab)

        this.worker = new Worker(path.join(__dirname, "./worker.js"))

        this.worker.on("error", error => {
            this.emit("container:error", error as any);
            console.error(error);
        })
        this.worker.on("message", (ev) => {
            switch (ev.command) {
                case From.ContainerReady:
                    this.container_id = ev.id
                    this.emit("container:ready", ev.id)
                    break;
                case From.LoadPlugin:
                    this.emit("plugin:loaded", ev.status)
                    break;
                case From.LoadProviderStatus:
                    this.emit("parse:provider", ev.status)
                    break;
                case From.LoadProgramStatus:
                    this.emit("parse:program", ev.status)
                    break;
                case From.RuntimeCommandsInt32:
                    if (ev.sab)
                        this.command_store = new CommandStore(new Int32Array(ev.sab), this.command_lock_int32)
                    break;
            }
        })
    }

    boot = (): Promise<string> => new Promise((resolve, reject) => {
        this.on("container:ready", ev => {
            if (ev) resolve(ev)
            else reject("Container failed to boot")
        })
        this.worker.postMessage({command: Send.Boot, pause_sab: this.pause_sab, command_lock_sab: this.command_lock_sab})
    })

    loadContainerParams = (params: ContainerParams) =>
        this.worker.postMessage({command: Send.LoadContainerParams, params})

    loadPlugin = (name: string, interval: number) =>
        this.worker.postMessage({command: Send.LoadPlugin, name, interval})

    loadProvider = (pack: Record<string, any>) => this.worker.postMessage({command: Send.LoadProvider, pack})

    loadProgram = (program: Record<string, any>) => this.worker.postMessage({command: Send.LoadProgram, program})

    clearProvider = () => this.worker.postMessage({command: Send.ClearProvider})

    clearProgram = () =>  this.worker.postMessage({command: Send.ClearProgram})

    start = (entry: string) => this.worker.postMessage({command: Send.Start, main: entry})

    pause = () => {
        if (this.command_store)
            this.command_store.pause()
    }

    resume = () => Atomics.notify(this.pause_int32, 0)

    stop = () => {
        if (this.command_store) {
            this.command_store.stop()
            this.resume()
        }
    }
    enableBreakpoint = (breakpoint: number) => {
        if (this.command_store)
            this.command_store.enableBreakpoint(breakpoint)
    }
    disableBreakpoint = (breakpoint: number) => {
        if (this.command_store)
            this.command_store.disableBreakpoint(breakpoint)
    }

    enableAllBreakpoints = () => {
        if (this.command_store)
            this.command_store.enableAllBreakpoints()
    }
    disableAllBreakpoints = () => {
        if (this.command_store)
            this.command_store.disableAllBreakpoints()
    }
}
