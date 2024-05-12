import {
    Stop,
    MonitorChange,
    UnitTest,
    SimulationStatus,
    ParseStatus,
    MonitorSchema,
    Stack,
    UnitTestStatus,
    UnitTestUpdateStatus
} from "../../vifsimlib.js";
import {Container} from "../boot/container.js";
import {VifEventEmitter} from "../event/event-emitter.js"

type Hooks = {
    "messages": (cb: string[]) => void;
    "warnings": (cb: string[]) => void;
    "error": (cb: Stop) => void;

    "monitor:changes": (cb: MonitorChange[]) => void;
    "breakpoint:current": (cb: number | undefined) => void;
    "unit-tests:statuses": (cb: UnitTestUpdateStatus[]) => void;

    "monitor:schemas": (cb: MonitorSchema[]) => void;
    "breakpoints": (cb: number[]) => void;
    "unit-tests": (cb: UnitTest[]) => void;
    "simulation:stack": (cb: Stack) => void;
    "simulation:entry-points": (cb: string[]) => void;

    "simulation:status": (cb: SimulationStatus) => void;
    "parse-provider:status": (cb: ParseStatus) => void;
    "parse-program:status": (cb: ParseStatus) => void;
}

interface AsyncExecutor {
    init: (container: Container) => Promise<{
        loadProvider(provider: Record<string, any>): Promise<void>
        loadProgram(program: Record<string, any>): Promise<void>
        startAndWaitUnitTests(entry: string): Promise<(UnitTest & UnitTestUpdateStatus)[]>
        start(entry: string): Promise<void>
        stop(): Promise<void>
        pause(): Promise<void>
        clearProvider(): Promise<void>
        clearProgram(): Promise<void>
        getMonitorSchemas(): MonitorSchema[]
        getMonitorSchemasAsObject(): MonitorSchema[]
        getBreakpoints(): number[]
        getUnitTests(): UnitTest[]
    }>
}

export class Plugin extends VifEventEmitter<Hooks> {
    private broadcast: BroadcastChannel
    private container: Container
    private name: string
    private interval: number
    private eventsMap: Record<number, [string, string]>;

    private unitTests: UnitTest[] = []
    private monitorSchemas: MonitorSchema[] = []
    private breakpoints: number[] = []

    constructor(name: string, interval: number) {
        super()
        this.name = name
        this.interval = interval
        this.eventsMap = {
            0: ["messages", "messages"],
            1: ["warnings", "warnings"],
            2: ["error", "error"],
            3: ["monitor:changes", "changes"],
            4: ["breakpoint:current", "current"],
            5: ["unit-tests:statuses", "statuses"],
            6: ["monitor:schemas", "schemas"],
            7: ["breakpoints", "breakpoints"],
            8: ["unit-tests", "unit_tests"],
            9: ["simulation:stack", "stack"],
            10: ["simulation:entry-points", "entries"],
            11: ["simulation:status", "status"],
            12: ["parse-provider:status", "status"],
            13: ["parse-program:status", "status"]
        }
    }

    public init = async (container: Container) => {
        return new Promise((resolve, reject) => {
            this.container = container;
            this.container.on("plugin:loaded", (ev) => {
                if (ev.status === 0) reject(`Could not add plugin ${this.name}, check your logs for more details`)

                this.broadcast = new BroadcastChannel(ev.name)

                this.broadcast.addEventListener("message", ev => {
                    const eventType = this.eventsMap[ev.data.type]
                    console.log(ev)
                    if (eventType) {
                        this.emit(eventType[0] as keyof Hooks, ev.data[eventType[1]])
                    }
                    resolve(0)
                })
            })
            this.container.loadPlugin(this.name, this.interval);
        })
    };

    private rejectOnError = (reject: (reason?: any) => void) => {
        this.on("error", error => reject(this.formatError(error)))
    }

    // Format Vif Stop error as Javascript error
    public formatError = (ev: Stop) => new class extends Error {
        original: Stop = ev

        constructor() {
            let err = `${ev.error}\n`
            if (ev.sim_stack) {
                err += "> Simulation stack:\n"
                ev.sim_stack.forEach(x => err += `  at -  ${x}\n`)
            }

            if (ev.id_stack)
                err += "File stack: \n"

            super(err)

            if (ev.id_stack) {
                let stack = ""
                ev.id_stack.forEach(x => stack += x)
                this.stack = stack
            }
        }
    }

    public getAsyncExecutor = (): AsyncExecutor => {
        const loadProvider = (provider: Record<string, any>): Promise<void> => {
            return new Promise((resolve, reject) => {
                this.rejectOnError(reject)
                this.on("parse-provider:status", status => {
                    if (status === ParseStatus.Loaded)
                        resolve()
                    else reject("Failed to load provider")
                })
                this.container.loadProvider(provider)
            })
        }

        const loadProgram = (program: Record<string, any>): Promise<void> => {
            return new Promise((resolve, reject) => {
                this.rejectOnError(reject)
                this.on("parse-program:status", status => {
                    if (status === ParseStatus.Loaded)
                        resolve()
                    else reject("Failed to load program")
                })
                this.container.loadProgram(program)
            })
        }

        const clearProvider = (): Promise<void> => {
            return new Promise((resolve, reject) => {
                this.rejectOnError(reject)
                this.on("parse-provider:status", status => resolve())
                this.container.clearProvider()
            })
        }

        const clearProgram = (): Promise<void> => {
            return new Promise((resolve, reject) => {
                this.rejectOnError(reject)
                this.on("parse-program:status", status => resolve())
                this.container.clearProgram()
            });
        }

        const start = (entry: string): Promise<void> => new Promise((resolve, reject) => {
            this.rejectOnError(reject)
            this.on("simulation:status", status => {
                if (status === SimulationStatus.Stop)
                    resolve()
            })
            this.container.start(entry)
        })

        const startAndWaitUnitTests = (entry: string): Promise<(UnitTest & UnitTestUpdateStatus)[]> => {
            if (!this.unitTests.length)
                return new Promise((resolve, reject) => reject("Unit tests list is empty"))
            return Promise.all<UnitTest & UnitTestUpdateStatus>(this.unitTests.map((a, i) =>
                new Promise((resolve, reject) => {
                        this.rejectOnError(reject)
                        this.on("simulation:status", status => {
                            if (status === 1) {
                                reject("Simulation was stopped before all units tests could be completed")
                            }
                        })
                        this.on("unit-tests:statuses", b => {
                            b.forEach(x => {
                                if (a.id === x.id) {
                                    switch (x.status) {
                                        case UnitTestStatus.Failed:
                                            resolve({
                                                description: a.description,
                                                id: a.id,
                                                status: UnitTestStatus.Failed,
                                                fail_message: x.fail_message
                                            })
                                            break;
                                        case UnitTestStatus.Succeed:
                                            resolve({
                                                description: a.description,
                                                id: a.id,
                                                status: UnitTestStatus.Succeed,
                                                fail_message: undefined // makes ts happy
                                            })
                                            break;
                                    }
                                }
                            })
                        })
                        // Cheap way to start once
                        if (i === 0) this.container.start(entry)
                    }
                )))
        }

        const getMonitorSchemas = () => this.monitorSchemas

        const getMonitorSchemasAsObject = () => {
            let root: Record<string, any> = {};
            return this.monitorSchemas
        }

        const getBreakpoints = () => this.breakpoints
        const getUnitTests = () => this.unitTests

        const stop = (): Promise<void> => {
            return new Promise((resolve, reject) => {
                this.on("simulation:status", status => {
                    if (status === SimulationStatus.Stop)
                        resolve()
                    else reject()
                })
                this.container.stop()
            })
        }

        const pause = (): Promise<void> => {
            return new Promise((resolve, reject) => {
                this.on("simulation:status", status => {
                    if (status === SimulationStatus.Pause)
                        resolve()
                    else reject()
                })
                this.container.pause()
            })
        }

        return {
            init: (container: Container) => {
                this.container = container
                if (!this.container.container_id)
                    return new Promise((resolve, reject) => reject("Container is not available, did you boot the vif-sim container first ?"))
                return new Promise((resolve, reject) => {
                    this.container.on("plugin:loaded", (ev) => {
                        if (ev.name === `${this.name}__${this.container.container_id}`) {
                            if (ev.status === 0) reject(`Could not add plugin ${this.name}, check your logs for more details`)
                            else {
                                this.broadcast = new BroadcastChannel(ev.name)
                                this.broadcast.addEventListener("message", ev => {
                                    const eventType = this.eventsMap[ev.data.type]
                                    if (eventType) {
                                        this.emit(eventType[0] as keyof Hooks, ev.data[eventType[1]])
                                        switch (eventType[0]) {
                                            case "unit-tests":
                                                this.unitTests = ev.data[eventType[1]]
                                                break;
                                            case "monitor:schemas":
                                                this.monitorSchemas = ev.data[eventType[1]]
                                                break;
                                            case "breakpoints":
                                                this.breakpoints = ev.data[eventType[1]]
                                                break;
                                            case "parse-program:status":
                                                if (ev.data[eventType[1]] === ParseStatus.Empty) {
                                                    this.unitTests = []
                                                    this.monitorSchemas = []
                                                    this.breakpoints = []
                                                }
                                                break;
                                        }
                                    }
                                })
                                resolve({
                                    loadProvider,
                                    loadProgram,
                                    start,
                                    stop,
                                    pause,
                                    startAndWaitUnitTests,
                                    clearProvider,
                                    clearProgram,
                                    getMonitorSchemas,
                                    getMonitorSchemasAsObject,
                                    getBreakpoints,
                                    getUnitTests,
                                })
                            }
                        }
                    })
                    this.container.loadPlugin(this.name, this.interval)
                })
            }
        }
    }
}
