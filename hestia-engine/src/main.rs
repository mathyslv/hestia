//! Hestia Engine - Windows Service bootstrap
//!
//! This binary hosts the core EDR engine as a Windows Service.
//! Service lifecycle (start/stop/pause/continue/shutdown) is handled via the
//! `windows-service` crate. Engine work can be plugged into the main loop.

#[cfg(windows)]
mod win_service {
    use anyhow::Result;
    use log::{error, info, warn};
    use std::ffi::OsString;
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;
    use windows_service::define_windows_service;
    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
    use windows_service::service_dispatcher;

    const SERVICE_NAME: &str = "EDR_Service";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    define_windows_service!(ffi_service_main, service_main);

    pub fn main() {
        // Initialize a basic logger (useful if running under console during dev)
        let _ = env_logger::try_init();

        // Start the service control dispatcher. This call blocks until service exits.
        if let Err(e) = service_dispatcher::start(SERVICE_NAME, ffi_service_main) {
            // If not running as a real service, just log and return an error.
            // This keeps behavior explicit and avoids surprises.
            error!("Failed to start service dispatcher: {e}");
        }
    }

    fn service_main(args: Vec<OsString>) {
        if let Err(e) = run_service(args) {
            error!("Service error: {e:?}");
        }
    }

    fn run_service(_args: Vec<OsString>) -> Result<()> {
        // Channel used to signal stop/shutdown from the control handler
        let (tx, rx) = mpsc::channel::<ServiceControl>();

        let handler = move |control_event: ServiceControl| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                ServiceControl::Stop
                | ServiceControl::Shutdown
                | ServiceControl::Pause
                | ServiceControl::Continue => {
                    let _ = tx.send(control_event);
                    ServiceControlHandlerResult::NoError
                }
                other => {
                    warn!("Unhandled control: {other:?}");
                    ServiceControlHandlerResult::NotImplemented
                }
            }
        };

        let status_handle = service_control_handler::register(SERVICE_NAME, handler)?;

        // Report start pending while initializing
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::StartPending,
            controls_accepted: ServiceControlAccept::STOP
                | ServiceControlAccept::SHUTDOWN
                | ServiceControlAccept::PAUSE_CONTINUE,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 1,
            wait_hint: Duration::from_secs(10),
            process_id: None,
        })?;

        // TODO: Initialize engine subsystems here
        info!("Hestia Engine initializing...");

        // Transition to Running
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP
                | ServiceControlAccept::SHUTDOWN
                | ServiceControlAccept::PAUSE_CONTINUE,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(0),
            process_id: None,
        })?;

        info!("Hestia Engine running");

        // Basic lifecycle state for pause/continue
        let mut paused = false;

        // Main loop: replace with real EDR work, timers, etc.
        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(ServiceControl::Stop) | Ok(ServiceControl::Shutdown) => {
                    info!("Stop/Shutdown received");
                    break;
                }
                Ok(ServiceControl::Pause) => {
                    if !paused {
                        paused = true;
                        info!("Paused");
                        status_handle.set_service_status(ServiceStatus {
                            service_type: SERVICE_TYPE,
                            current_state: ServiceState::Paused,
                            controls_accepted: ServiceControlAccept::STOP
                                | ServiceControlAccept::SHUTDOWN
                                | ServiceControlAccept::PAUSE_CONTINUE,
                            exit_code: ServiceExitCode::Win32(0),
                            checkpoint: 0,
                            wait_hint: Duration::from_secs(0),
                            process_id: None,
                        })?;
                    }
                }
                Ok(ServiceControl::Continue) => {
                    if paused {
                        paused = false;
                        info!("Continued");
                        status_handle.set_service_status(ServiceStatus {
                            service_type: SERVICE_TYPE,
                            current_state: ServiceState::Running,
                            controls_accepted: ServiceControlAccept::STOP
                                | ServiceControlAccept::SHUTDOWN
                                | ServiceControlAccept::PAUSE_CONTINUE,
                            exit_code: ServiceExitCode::Win32(0),
                            checkpoint: 0,
                            wait_hint: Duration::from_secs(0),
                            process_id: None,
                        })?;
                    }
                }
                Ok(_) => {}
                Err(RecvTimeoutError::Timeout) => {
                    // Periodic work tick; skip when paused
                    if !paused {
                        // TODO: Insert engine recurring tasks here
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    warn!("Control channel disconnected; stopping");
                    break;
                }
            }
        }

        // Signal stop pending during shutdown/cleanup
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::StopPending,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 1,
            wait_hint: Duration::from_secs(10),
            process_id: None,
        })?;

        // TODO: Cleanup subsystems here
        info!("Hestia Engine stopped");

        // Final state: Stopped
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(0),
            process_id: None,
        })?;

        Ok(())
    }
}

#[cfg(windows)]
fn main() {
    win_service::main()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("hestia-engine runs only on Windows (service host)");
}
