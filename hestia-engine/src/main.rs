//! Hestia Engine - Windows Service bootstrap
//!
//! This binary hosts the core EDR engine as a Windows Service.
//! Service lifecycle (start/stop/pause/continue/shutdown) is handled via the
//! `windows-service` crate. Engine work can be plugged into the main loop.

#[cfg(windows)]
mod win_service {
    use anyhow::{anyhow, Result};
    use clap::Parser;
    use std::ffi::OsString;
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;
    use tracing::{error, info, warn};
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    use windows_service::define_windows_service;
    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
    use windows_service::service_dispatcher;

    use tracing_etw::EtwLayer;

    #[derive(Parser, Debug, Clone, Copy)]
    #[command(
        name = "hestia-engine",
        about = "Hestia detection engine Windows service",
        author,
        version
    )]
    pub struct Cli {
        /// Run the engine in the foreground with console logging instead of ETW
        #[arg(long)]
        pub foreground: bool,
    }

    const SERVICE_NAME: &str = "EDR_Service";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    define_windows_service!(ffi_service_main, service_main);

    pub fn main(args: Cli) -> Result<()> {
        if args.foreground {
            run_engine(RuntimeMode::Foreground, Vec::new())
        } else {
            service_dispatcher::start(SERVICE_NAME, ffi_service_main)
                .map_err(|err| anyhow!("Failed to start service dispatcher: {err}"))
        }
    }

    fn service_main(args: Vec<OsString>) {
        if let Err(e) = run_engine(RuntimeMode::Service, args) {
            error!("Service error: {e:?}");
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum RuntimeMode {
        Service,
        Foreground,
    }

    type StatusHandle = Option<service_control_handler::ServiceStatusHandle>;

    fn init_tracing(mode: RuntimeMode) -> Result<Option<tracing_etw::EtwLayerGuard>> {
        match mode {
            RuntimeMode::Foreground => {
                tracing_subscriber::registry()
                    .with(tracing_subscriber::fmt::layer().with_target(true))
                    .try_init()?;
                Ok(None)
            }
            RuntimeMode::Service => {
                let (layer, guard) = EtwLayer::new("HestiaEngine")?;
                tracing_subscriber::registry().with(layer).try_init()?;
                Ok(Some(guard))
            }
        }
    }

    fn set_status(handle: &StatusHandle, status: ServiceStatus) -> windows_service::Result<()> {
        if let Some(handle) = handle {
            handle.set_service_status(status)?;
        }
        Ok(())
    }

    fn run_engine(mode: RuntimeMode, _args: Vec<OsString>) -> Result<()> {
        let (tx, rx) = mpsc::channel::<ServiceControl>();

        let status_handle = if mode == RuntimeMode::Service {
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

            Some(service_control_handler::register(SERVICE_NAME, handler)?)
        } else {
            None
        };

        let _guard = init_tracing(mode)?;

        // Report start pending while initializing
        set_status(
            &status_handle,
            ServiceStatus {
                service_type: SERVICE_TYPE,
                current_state: ServiceState::StartPending,
                controls_accepted: ServiceControlAccept::STOP
                    | ServiceControlAccept::SHUTDOWN
                    | ServiceControlAccept::PAUSE_CONTINUE,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 1,
                wait_hint: Duration::from_secs(10),
                process_id: None,
            },
        )?;

        // TODO: Initialize engine subsystems here
        info!("Hestia Engine initializing...");

        // Transition to Running
        set_status(
            &status_handle,
            ServiceStatus {
                service_type: SERVICE_TYPE,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP
                    | ServiceControlAccept::SHUTDOWN
                    | ServiceControlAccept::PAUSE_CONTINUE,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::from_secs(0),
                process_id: None,
            },
        )?;

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
                        set_status(
                            &status_handle,
                            ServiceStatus {
                                service_type: SERVICE_TYPE,
                                current_state: ServiceState::Paused,
                                controls_accepted: ServiceControlAccept::STOP
                                    | ServiceControlAccept::SHUTDOWN
                                    | ServiceControlAccept::PAUSE_CONTINUE,
                                exit_code: ServiceExitCode::Win32(0),
                                checkpoint: 0,
                                wait_hint: Duration::from_secs(0),
                                process_id: None,
                            },
                        )?;
                    }
                }
                Ok(ServiceControl::Continue) => {
                    if paused {
                        paused = false;
                        info!("Continued");
                        set_status(
                            &status_handle,
                            ServiceStatus {
                                service_type: SERVICE_TYPE,
                                current_state: ServiceState::Running,
                                controls_accepted: ServiceControlAccept::STOP
                                    | ServiceControlAccept::SHUTDOWN
                                    | ServiceControlAccept::PAUSE_CONTINUE,
                                exit_code: ServiceExitCode::Win32(0),
                                checkpoint: 0,
                                wait_hint: Duration::from_secs(0),
                                process_id: None,
                            },
                        )?;
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
        set_status(
            &status_handle,
            ServiceStatus {
                service_type: SERVICE_TYPE,
                current_state: ServiceState::StopPending,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 1,
                wait_hint: Duration::from_secs(10),
                process_id: None,
            },
        )?;

        // TODO: Cleanup subsystems here
        info!("Hestia Engine stopped");

        // Final state: Stopped
        set_status(
            &status_handle,
            ServiceStatus {
                service_type: SERVICE_TYPE,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::from_secs(0),
                process_id: None,
            },
        )?;

        Ok(())
    }
}

#[cfg(windows)]
fn main() {
    let args = win_service::Cli::parse();
    if let Err(error) = win_service::main(args) {
        eprintln!("{error:?}");
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("hestia-engine runs only on Windows (service host)");
}
