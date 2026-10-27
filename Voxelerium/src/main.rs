// Copyright (C) 2026 XYZabc123456789
//
// This file is part of Voxelerium.
//
// Voxelerium is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Voxelerium is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Voxelerium.  If not, see <https://www.gnu.org/licenses/>.

//main.rs
use core_affinity2::*;
use rayon::*;
use renderer::*;
use std::sync::Arc;
use std::thread;
use tokio::runtime::{Builder, Handle};
use tokio::sync::oneshot;
use tracing::{debug, info, instrument, trace, warn};
use tracing_subscriber::EnvFilter;
use winit::application::ApplicationHandler;
use winit::event::*;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::Key;
use winit::window::{Window, WindowId};

mod app;
use app::*;

mod input;
use input::*;

mod ui;
#[allow(unused)]
use ui::*;

const COPYRIGHT: &str = "Voxelerium  Copyright (C) 2026  XYZabc123456789
This program comes with ABSOLUTELY NO WARRANTY; for details see the about box.
This is free software, and you are welcome to redistribute it
under certain conditions; see the about box for details.";

fn main() {
    println!("{}", COPYRIGHT);
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
    core_ids[0]
        .set_affinity()
        .expect("Could not set CPU affinity");
    trace!("Main thread affinity: CPU0");
    trace!("Creating Threadpool");
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .start_handler(move |thread_id| {
            let core_ids = get_core_ids().unwrap();
            let core_id = core_ids[(thread_id + 2) % core_ids.len()];
            core_id.set_affinity().expect("Could not set CPU affinity");
            debug!(
                thread_id,
                cpu_id = core_id.0,
                "worker CPU affinity configured"
            );
        })
        .build_global()
        .expect("Could not create threads");
    debug!("Threadpool created");
    rayon::scope(|_| {
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
