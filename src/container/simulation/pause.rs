use std::time::Instant;
use ansi_term::Color::Yellow;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::container::SimulationStatus;
use crate::container::error::error::Stop;

pub fn pause_simulation(channel: &Broadcast, id: Option<u64>) -> Result<(), Stop> {
    channel.add_message(
        &Yellow.paint("[Pause] Simulation paused").to_string());
    let earlier = Instant::now();

    channel.push_cycle_stack();
    channel.set_simulation_status(&SimulationStatus::Pause);

    #[cfg(not(target_arch = "wasm32"))]
    channel.add_warning("Pause is not available on OS targets.");

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(id) = id {
            channel.activate_breakpoint(id);
        }
        channel.publish();
        js_sys::Atomics::wait(&channel.get_pause_int32(), 0, 1).unwrap_throw();

        (*DELAYED_TIMERS.lock().unwrap())
            .iter_mut()
            .for_each(|(_ptr, dur)| {
                *dur += Instant::now().duration_since(earlier);
            });

        channel.add_message(&Green.paint("[Pause] Simulation resumed").to_string());
        channel.set_simulation_status(&SimulationStatus::Start);
        if let Some(id) = id {
            channel.disable_breakpoint();
        }
        channel.publish();

        let stop = read_sab_commands(&channel);
        if stop {
            return Err(Stop::new("Manual stop before cycle end".into(), &None, id))
        }
    }
    Ok(())
}

pub fn enableBreakpoint(channel: &Broadcast, bp: u64) {
    channel.add_breakpoint(bp);
    channel.add_message(&format!("Enabled breakpoint {}", bp))
}

pub fn disableBreakpoint(channel: &Broadcast, bp: u64) {
    channel.remove_breakpoint(bp);
    channel.add_message(&format!("Disabled breakpoint {}", bp))
}