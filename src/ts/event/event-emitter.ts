type ArgumentTypes<F extends Function> = F extends (...args: infer A) => any ? A : never;
type EventEmitterTypes = Record<string, (...args: any[]) => void>

export class VifEventEmitter<T extends EventEmitterTypes> {
    private readonly events: Record<keyof T, T[keyof T][]> = {} as Record<keyof T, T[keyof T][]>;

    constructor() {}

    public on<Y extends keyof T>(event: Y, listener: T[Y]): () => void {
        if(typeof this.events[event] !== 'object') this.events[event] = [];

        this.events[event].push(listener)
        return () => this.removeListener(event, listener)
    }

    public removeListener<Y extends keyof T>(event: Y, listener: T[Y]): void {
        if(typeof this.events[event] !== 'object') return;

        const idx: number = this.events[event].indexOf(listener);
        if(idx > -1) this.events[event].splice(idx, 1);
    }

    public removeAllListeners(): void {
        Object.keys(this.events).forEach((event: string) =>
            this.events[event].splice(0, this.events[event].length)
        );
    }

    public emit<Y extends keyof T>(event: Y, ...args: ArgumentTypes<T[Y]>): void {
        if(typeof this.events[event] !== 'object') return;

        this.events[event].forEach(listener => listener.apply(this, args));
    }

    public once<Y extends keyof T>(event: Y, listener: T[Y]): void {
        //@ts-expect-error TS2345
        const remove: (() => void) = this.on(event, (...args: ArgumentTypes<T[Y]>) => {
            remove();
            listener.apply(this, args);
        });
    }
}