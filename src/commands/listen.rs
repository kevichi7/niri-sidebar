use crate::Ctx;
use crate::commands::reorder;
use crate::niri::connect;
use crate::state::{get_cache_dir, load_state, save_state};
use anyhow::Result;
use fslock::LockFile;
use niri_ipc::{Event, Request};

pub fn listen(mut ctx: Ctx) -> Result<()> {
    let _ = ctx.socket.send(Request::EventStream)?;
    let mut read_event = ctx.socket.read_events();
    println!("niri-sidebar: Listening for window events...");

    while let Ok(event) = read_event() {
        if let Event::WindowClosed { id } = event {
            handle_close_event(id)?;
        }
    }

    Ok(())
}

fn handle_close_event(closed_id: u64) -> Result<()> {
    let mut lock_path = get_cache_dir()?;
    lock_path.push("instance.lock");
    let mut lock_file = LockFile::open(&lock_path)?;
    lock_file.lock()?;

    let mut state = load_state()?;
    let config = crate::config::load_config();

    if let Some(index) = state.windows.iter().position(|(id, _, _)| *id == closed_id) {
        println!("Sidebar window {} closed. Reordering...", closed_id);

        state.windows.remove(index);
        save_state(&state)?;

        let action_socket = connect()?;
        let mut action_ctx = Ctx {
            state,
            config,
            socket: action_socket,
        };

        if let Err(e) = reorder(&mut action_ctx) {
            eprintln!("Failed to reorder: {}", e);
        }
    }

    Ok(())
}
