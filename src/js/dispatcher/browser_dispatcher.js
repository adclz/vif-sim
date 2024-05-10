export class BrowserDispatcher {
    server_id
    dispatcher_worker
    plugins = []

    constructor(server_id) {
        this.server_id = server_id
        const dispatcher_string = dispatcher.toString()
        this.dispatcher_worker = new Worker(URL.createObjectURL(new Blob([dispatcher_string.substring(dispatcher_string.indexOf("{") + 1, dispatcher_string.lastIndexOf("}"))], {type: 'application/javascript'})))
        this.dispatcher_worker.addEventListener("error", error => console.error(error))
        this.dispatcher_worker.postMessage({type: 0, server_id})
        console.log(`Browser dispatcher is launched with id ${this.server_id}`)
    }

    addPlugin(interval, name) {
        if (this.plugins.includes(name)) {
            console.log(`Could not add plugin ${name}: A plugin already exists with the same name`)
            return { name: `${name}__${this.server_id}`, status: 0 }
        }
        this.plugins.push(name)
        this.dispatcher_worker.postMessage({type: 1, interval, name})
        return { name: `${name}__${this.server_id}`, status: 1 }
    }

    publishStore = (store) => {
        const received_store = {
            stack: store.get_stack,
            messages: store.get_messages,
            warnings: store.get_warnings,
            error: store.get_error,
            monitor_schemas: store.get_monitor_schemas,
            monitor_changes: store.get_monitor_changes,
            breakpoints: store.get_breakpoints,
            breakpoints_statuses: store.get_breakpoints_statuses,
            unit_tests: store.get_unit_tests,
            unit_tests_statuses: store.get_unit_tests_statuses,
            entry_points: store.get_entry_points,
            simulation_status: store.get_simulation_status,
            parse_provider_status: store.get_parse_provider_status,
            parse_program_status: store.get_parse_program_status
        }

        const filtered_store = {}
        if (received_store.stack) filtered_store.stack = received_store.stack
        if (received_store.messages) filtered_store.messages = received_store.messages
        if (received_store.warnings) filtered_store.warnings = received_store.warnings
        if (received_store.error) filtered_store.error = received_store.error
        if (received_store.monitor_schemas) filtered_store.monitor_schemas = received_store.monitor_schemas.map(x => ({
            path: x.get_path,
            value: x.get_value
        }))
        if (received_store.monitor_changes) filtered_store.monitor_changes = received_store.monitor_changes.map(x => ({
            id: x.get_id,
            value: x.get_value
        }))
        if (received_store.breakpoints) filtered_store.breakpoints = received_store.breakpoints.map(x => ({
            id: x.get_id,
            status: x.status
        }))
        if (received_store.breakpoints_statuses) filtered_store.breakpoints_statuses = received_store.breakpoints_statuses.map(x => ({
            id: x.get_id,
            status: x.get_status
        }))
        if (received_store.unit_tests) filtered_store.unit_tests = received_store.unit_tests.map(x => ({
            id: x.get_id,
            description: x.get_description,
            status: x.status
        }))
        if (received_store.unit_tests_statuses) filtered_store.unit_tests_statuses = received_store.unit_tests_statuses.map(x => ({
            id: x.get_id,
            status: x.get_status,
            fail_message: x.get_fail_message
        }))
        if (received_store.entry_points) filtered_store.entry_points = received_store.entry_points
        if (typeof received_store.simulation_status !== "undefined") filtered_store.simulation_status = received_store.simulation_status
        if (typeof received_store.parse_provider_status !== "undefined") filtered_store.parse_provider_status = received_store.parse_provider_status
        if (typeof received_store.parse_program_status !== "undefined") filtered_store.parse_program_status = received_store.parse_program_status

        this.dispatcher_worker.postMessage({
            type: 2, store: filtered_store
        })
    }
}

const dispatcher = function () {
    let plugins = []
    let server_id

    self.addEventListener("message", e => {
        switch (e.data.type) {
            case 0: {
                server_id = e.data.server_id
                break;
            }

            // Add plugin
            case 1: {
                const plugin_string = plugin.toString()
                const plugin_worker = new Worker(URL.createObjectURL(new Blob([plugin_string.substring(plugin_string.indexOf("{") + 1, plugin_string.lastIndexOf("}"))], {type: 'application/javascript'})))
                plugin_worker.addEventListener("error", error => console.error(error))
                plugin_worker.postMessage({type: 0, id: server_id, name: e.data.name, interval: e.data.interval,})
                plugins.push(plugin_worker)
                console.log(`New plugin registered on dispatcher with name ${e.data.name} and interval ${e.data.interval}`)
                break;
            }

            // Store
            case 2: {
                plugins.forEach(plugin => plugin.postMessage({type: 2, store: e.data.store}))
            }
        }
    })

    const plugin = function () {
        let broadcast
        let store = null

        let breakpoint_statuses_memos = []
        let unit_tests_statuses_memos = []

        let parse_provider_status_memo = null
        let parse_program_status_memo = null
        let simulation_status_memo = null

        self.addEventListener("message", e => {
            switch (e.data.type) {
                // Init
                case 0:
                    checkBroadcast(e.data.id, e.data.name, e.data.interval)
                    break;
                // Store
                case 2:
                    store = e.data.store
                    if (typeof store.unit_tests_statuses !== "undefined")
                        unit_tests_statuses_memos.push(...store.unit_tests_statuses)

                    if (typeof store.breakpoints_statuses !== "undefined")
                        breakpoint_statuses_memos.push(...store.breakpoints_statuses)

                    if (typeof store.parse_provider_status !== "undefined")
                        parse_provider_status_memo = store.parse_provider_status

                    if (typeof store.parse_program_status !== "undefined")
                        parse_program_status_memo = store.parse_program_status

                    if (typeof store.simulation_status !== "undefined")
                        simulation_status_memo = store.simulation_status

                    break;
            }
        })


        const checkBroadcast = (id, name, interval) => {
            if (!broadcast) {
                broadcast = new BroadcastChannel(`${name}__${id}`)
                console.log(`Broadcasting on channel ${name}__${id}`)
                setInterval(() => {
                    if (store) {
                        const store_copy = Object.assign({}, store)
                        store = null

                        if (store_copy.messages) broadcast.postMessage({type: 0, messages: store_copy.messages})

                        if (store_copy.warnings) broadcast.postMessage({type: 1, warnings: store_copy.warnings})

                        if (store_copy.error) broadcast.postMessage({type: 2, error: store_copy.error})

                        // Changes

                        if (store_copy.monitor_changes) broadcast.postMessage({
                            type: 3, changes: store_copy.monitor_changes
                        })

                        if (breakpoint_statuses_memos.length) {
                            broadcast.postMessage({
                                type: 4, statuses: breakpoint_statuses_memos
                            })
                            breakpoint_statuses_memos = []
                        }

                        if (unit_tests_statuses_memos.length) {
                            broadcast.postMessage({
                                type: 5, statuses: unit_tests_statuses_memos
                            })
                            unit_tests_statuses_memos = []
                        }

                        // Schemas

                        if (store_copy.monitor_schemas) broadcast.postMessage({
                            type: 6, schemas: store_copy.monitor_schemas
                        })

                        if (store_copy.breakpoints) broadcast.postMessage({
                            type: 7, breakpoints: store_copy.breakpoints
                        })

                        if (store_copy.unit_tests) broadcast.postMessage({
                            type: 8, unit_tests: store_copy.unit_tests
                        })

                        if (store_copy.stack) broadcast.postMessage({type: 9, stack: store_copy.stack})

                        if (store_copy.entry_points) broadcast.postMessage({type: 10, entries: store_copy.entry_points})

                        // Statuses

                        if (simulation_status_memo !== null) {
                            broadcast.postMessage({type: 11, status: simulation_status_memo})
                            simulation_status_memo = null
                        }
                        if (parse_provider_status_memo !== null) {
                            broadcast.postMessage({type: 12, status: parse_provider_status_memo})
                            parse_provider_status_memo = null
                        }
                        if (parse_program_status_memo !== null) {
                            broadcast.postMessage({type: 13, status: parse_program_status_memo})
                            parse_program_status_memo = null
                        }
                    }
                }, interval)
            }
        }
    }
}
