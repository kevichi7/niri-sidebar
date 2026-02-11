use crate::{Ctx, NiriClient};
use anyhow::Result;
use niri_ipc::{Action, Window, WorkspaceReferenceArg};

pub fn move_from<C: NiriClient>(ctx: &mut Ctx<C>, workspace: u64) -> Result<()> {
    let active_workspace = ctx.socket.get_active_workspace()?.id;
    let windows = ctx.socket.get_windows()?;

    let windows_on_ws: Vec<_> = windows
        .iter()
        .filter(|w| {
            w.workspace_id == Some(workspace)
                && ctx.state.windows.iter().any(|&(id, _, _)| id == w.id)
        })
        .collect();

    move_to(ctx, windows_on_ws, active_workspace)?;

    Ok(())
}

pub fn move_to<C: NiriClient>(ctx: &mut Ctx<C>, windows: Vec<&Window>, to_ws: u64) -> Result<()> {
    for w in windows {
        ctx.socket.send_action(Action::MoveWindowToWorkspace {
            window_id: Some(w.id),
            reference: WorkspaceReferenceArg::Id(to_ws),
            focus: false,
        })?;
    }

    Ok(())
}
