use crate::Ctx;
use crate::commands::reorder;
use crate::niri::get_windows;
use crate::state::save_state;
use anyhow::{Context, Result};
use niri_ipc::{Action, Request};

pub fn close(ctx: &mut Ctx) -> Result<()> {
    let windows = get_windows(&mut ctx.socket)?;
    let focused = windows
        .iter()
        .find(|w| w.is_focused)
        .context("No window focused")?;

    if let Some(index) = ctx
        .state
        .windows
        .iter()
        .position(|(id, _, _)| *id == focused.id)
    {
        ctx.state.windows.remove(index);
        save_state(&ctx.state)?;
    }

    let action = Action::CloseWindow {
        id: Some(focused.id),
    };
    let _ = ctx.socket.send(Request::Action(action))?;
    reorder(ctx)?;

    Ok(())
}
