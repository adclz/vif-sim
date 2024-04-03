export class CommandStore {
    RuntimeCommandsInt32: Int32Array
    CommandLock: Int32Array
    LastIndex: number = 0
    Stop: null | number = null
    Pause: null | number = null
    EnableAllBreakpoints: null | number = null
    DisableAllBreakpoints: null | number = null

    BreakpointsEnable: Record<number, number | null> = {}
    BreakpointsDisable: Record<number, number | null> = {}

    constructor(RuntimeCommandsInt32: Int32Array, CommandLock: Int32Array) {
        this.RuntimeCommandsInt32 = RuntimeCommandsInt32
        this.CommandLock = CommandLock
    }

    checkReset = () => {
        if (!this.RuntimeCommandsInt32.at(0)) {
            this.LastIndex = 0
            this.Stop = null
            this.Pause = null
            this.EnableAllBreakpoints = null
            this.DisableAllBreakpoints = null
            this.BreakpointsEnable = {}
            this.BreakpointsDisable = {}
        }
    }

    stop = async () => {
        await Atomics.waitAsync(this.CommandLock, 0, 1).value
        this.checkReset()
        if (this.Stop === null) {
            this.RuntimeCommandsInt32[this.LastIndex] = 1
            this.RuntimeCommandsInt32[this.LastIndex + 1] = 0
            this.Stop = this.LastIndex
            this.LastIndex += 2
        }
    }

    pause = async () => {
        await Atomics.waitAsync(this.CommandLock, 0, 1).value
        this.checkReset()
        if (this.Pause === null) {
            this.RuntimeCommandsInt32[this.LastIndex] = 2
            this.RuntimeCommandsInt32[this.LastIndex + 1] = 0
            this.Pause = this.LastIndex
            this.LastIndex += 2
        }
    }

    enableAllBreakpoints = async () => {
        await Atomics.waitAsync(this.CommandLock, 0, 1).value
        this.checkReset()
        if (this.EnableAllBreakpoints === null) {
            this.RuntimeCommandsInt32[this.LastIndex] = 3
            this.RuntimeCommandsInt32[this.LastIndex + 1] = 0
            this.EnableAllBreakpoints = this.LastIndex
            this.LastIndex += 2
        }
    }

    disableAllBreakpoints = async () => {
        await Atomics.waitAsync(this.CommandLock, 0, 1).value
        this.checkReset()
        if (this.DisableAllBreakpoints === null) {
            this.RuntimeCommandsInt32[this.LastIndex] = 4
            this.RuntimeCommandsInt32[this.LastIndex + 1] = 0
            this.DisableAllBreakpoints = this.LastIndex
            this.LastIndex += 2
        }
    }

    enableBreakpoint = async (breakpoint: number) => {
        await Atomics.waitAsync(this.CommandLock, 0, 1).value
        this.checkReset()
        if (typeof this.BreakpointsEnable[breakpoint] === "undefined") {
            this.RuntimeCommandsInt32[this.LastIndex] = 5
            this.RuntimeCommandsInt32[this.LastIndex + 1] = breakpoint
            this.BreakpointsEnable[breakpoint] = this.LastIndex
            this.LastIndex += 2
        }
    }

    disableBreakpoint = async (breakpoint: number) => {
        await Atomics.waitAsync(this.CommandLock, 0, 1).value
        this.checkReset()
        if (typeof this.BreakpointsDisable[breakpoint] === "undefined") {
            this.RuntimeCommandsInt32[this.LastIndex] = 6
            this.RuntimeCommandsInt32[this.LastIndex + 1] = breakpoint
            this.BreakpointsDisable[breakpoint] = this.LastIndex
            this.LastIndex += 2
        }
    }
}