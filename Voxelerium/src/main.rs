//main.rs
#![allow(unused)]
#![allow(type_alias_bounds)]
use ::utils::Gresult;
use core_affinity2::*;
use rayon::*;
use renderer::*;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tokio::runtime::{Builder, Handle};
use tokio::sync::oneshot;
use tracing::{debug, error, info, instrument, trace, warn};
use tracing_subscriber::EnvFilter;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Debug)]
struct App {
    window: Option<Arc<Window>>,
    focus: bool,
    gpu: Option<Gpu>,
    handle: Handle,
}

impl App {
    fn new(handle: Handle) -> Self {
        Self {
            window: None,
            focus: false,
            gpu: None,
            handle,
        }
    }
}

impl ApplicationHandler for App {
    #[instrument(
        skip_all,
        name = "window_creation"
        fields(
            window_title = "Voxelerium",
            gpu_initialized = tracing::field::Empty
        )
    )]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        debug!("Creating Window...");
        self.window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Voxelerium"))
                .expect("Could not create window"),
        ));
        trace!("Window creared");
        debug!("Setting up GPU...");
        self.gpu = Some(
            self.handle
                .block_on(Gpu::init(
                    self.window.as_ref().unwrap().clone(),
                    self.handle.clone(),
                ))
                .expect("GPU initialization failed"),
        );
        tracing::Span::current().record("gpu_initialized", true);
        trace!("GPU Setup Complete")
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Closing...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let _span = tracing::trace_span!("render_frame", focused = self.focus).entered();
                let r = self
                    .gpu
                    .as_mut()
                    .expect("GPU not initialised")
                    .render(self.focus);
                match r {
                    Ok(_) => {}
                    Err(e) => panic!("{:#?}", e),
                }
                self.window
                    .as_mut()
                    .expect("Window not Initialised")
                    .request_redraw();
            }
            WindowEvent::Resized(s) => {
                self.gpu.as_mut().expect("GPU not initialised").resize(s);
            }
            WindowEvent::Focused(f) => {
                self.focus = f;
            }
            _ => (),
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {}
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new(if cfg!(debug_assertions) {
                "trace,wgpu=info,naga=info"
            } else {
                "info"
            })
        }))
        .init();
    trace!("getting CPU affinity");
    let core_ids = get_core_ids().expect("Could not set CPU affinity");
    if core_ids.len() <= 2 {
        panic!("Please use a CPU with at least three cores");
    }
    debug!("cores: {}", core_ids.len());
    let num_threads = core_ids.len() - 2;
    core_ids[0].set_affinity();
    trace!("Main thread affinity: CPU0");
    trace!("Creating Threadpool");
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .start_handler(move |thread_id| {
            let core_ids = get_core_ids().unwrap();
            let core_id = core_ids[(thread_id + 2) % core_ids.len()];
            core_id.set_affinity();
            debug!(
                thread_id,
                cpu_id = core_id.0,
                "worker CPU affinity configured"
            );
        })
        .build_global()
        .expect("Could not create threads");
    debug!("Threadpool created");
    rayon::scope(|s| {
        info!("Now utilising {} worker threads", num_threads);
    });
    trace!("creating shutdown & ready channels");
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
    trace!("channels created");
    trace!("setting up async thread");
    let tokio_thread = thread::Builder::new()
        .name("async".into())
        .spawn(move || {
            trace!("Building Tokio runtime");
            let runtime = Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create async runtime");
            trace!("Tokio runtime initialized");
            let handle = runtime.handle().clone();
            ready_tx.send(handle).expect("Caller dropped");
            runtime.block_on(async {
                debug!("Async runtime started");
                let _ = shutdown_rx.await;
                debug!("Async runtime shutting down");
            });
        })
        .expect("Failed to spawn async thread");
    debug!("async thread created");
    trace!("getting async handle");
    let handle = ready_rx.recv().expect("async thread failed");
    trace!("recieved async handle");
    debug!("creating event loop");
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(handle);
    debug!("Starting event loop");
    let r = event_loop.run_app(&mut app);
    debug!("Event loop over");
    let _ = shutdown_tx.send(());
    tokio_thread.join().expect("async thread panicked");
    match r {
        Ok(_) => {
            info!("TERMINATED SUCCESSFULLY")
        }
        Err(e) => {
            panic!("{:#?}", e)
        }
    }
}
