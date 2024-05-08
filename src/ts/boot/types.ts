import {ParseStatus, ContainerParams} from "../../vifsimlib.js";

export enum Send {
    Boot,
    LoadContainerParams,
    LoadPlugin,
    LoadProvider,
    LoadProgram,
    Start,
    Stop,
    ClearProgram,
    ClearProvider,
}

export type ToCommand =
    { command: Send.LoadProvider, pack: Record<string, any> } |
    { command: Send.LoadContainerParams, params: ContainerParams } |
    { command: Send.Boot, pause_sab: SharedArrayBuffer, command_lock_sab: SharedArrayBuffer } |
    { command: Send.LoadPlugin, name: string, interval: number } |
    { command: Send.ClearProgram } |
    { command: Send.ClearProvider } |
    { command: Send.Start, main: string } |
    { command: Send.LoadProgram, program: Record<string, any> }

export enum From {
    ContainerReady,
    LoadProviderStatus,
    LoadProgramStatus,
    LoadPlugin,
    RuntimeCommandsInt32
}

export type FromCommand =
    { command: From.ContainerReady, id: string } |
    { command: From.LoadProviderStatus, status: ParseStatus } |
    { command: From.LoadProgramStatus, status: ParseStatus } |
    { command: From.LoadPlugin, status: { name: string, status: 0 | 1 } } |
    { command: From.RuntimeCommandsInt32, sab: Int32Array }