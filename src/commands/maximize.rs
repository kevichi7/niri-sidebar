use crate::commands::reorder;
use crate::niri::NiriClient;
use crate::state::save_state;
use crate::{Ctx, config::SidebarPosition};
use anyhow::Result;
use niri_ipc::{Action, SizeChange};

pub fn maximize<C: NiriClient>(ctx: &mut Ctx<C>) -> Result<()> {
    let focused_id = ctx.socket.get_active_window()?.id;

    if !ctx.state.windows.iter().any(|w| w.id == focused_id) {
        return Ok(());
    }

    ctx.state.maximized_window_id = match ctx.state.maximized_window_id {
        Some(id) if id == focused_id => None,
        _ => Some(focused_id),
    };

    if ctx.state.maximized_window_id.is_none() {
        restore_sidebar_window_sizes(ctx)?;
    }

    save_state(&ctx.state, &ctx.cache_dir)?;
    reorder(ctx)?;
    Ok(())
}

pub(crate) fn restore_sidebar_window_sizes<C: NiriClient>(ctx: &mut Ctx<C>) -> Result<()> {
    let windows = ctx.socket.get_windows()?;
    let current_ws = ctx.socket.get_active_workspace()?.id;

    for window in windows.iter().filter(|w| {
        w.workspace_id == Some(current_ws) && ctx.state.windows.iter().any(|ws| ws.id == w.id)
    }) {
        let (target_width, target_height) = crate::window_rules::resolve_window_size(
            &ctx.config.window_rule,
            window,
            ctx.config.geometry.width,
            ctx.config.geometry.height,
        );

        match ctx.config.interaction.position {
            SidebarPosition::Left | SidebarPosition::Right => {
                let _ = ctx.socket.send_action(Action::SetWindowHeight {
                    change: SizeChange::SetFixed(target_height),
                    id: Some(window.id),
                });
            }
            SidebarPosition::Top | SidebarPosition::Bottom => {
                let _ = ctx.socket.send_action(Action::SetWindowWidth {
                    change: SizeChange::SetFixed(target_width),
                    id: Some(window.id),
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SidebarPosition;
    use crate::state::{AppState, WindowState};
    use crate::test_utils::{MockNiri, mock_config, mock_window};
    use niri_ipc::Action;
    use tempfile::tempdir;

    #[test]
    fn test_maximize_toggle_sets_and_unsets_maximized_window() {
        let temp_dir = tempdir().unwrap();
        let win = mock_window(10, true, true, 1, Some((1.0, 2.0)));
        let mock = MockNiri::new(vec![win]);

        let mut state = AppState::default();
        state.windows.push(WindowState {
            id: 10,
            width: 1000,
            height: 800,
            is_floating: true,
            position: Some((1.0, 2.0)),
        });

        let mut ctx = Ctx {
            state,
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        maximize(&mut ctx).expect("maximize failed");
        assert_eq!(ctx.state.maximized_window_id, Some(10));

        ctx.socket.sent_actions.clear();
        maximize(&mut ctx).expect("maximize toggle-off failed");
        assert_eq!(ctx.state.maximized_window_id, None);
        assert!(
            ctx.socket
                .sent_actions
                .iter()
                .any(|a| matches!(a, Action::SetWindowHeight { id: Some(10), .. }))
        );
    }

    #[test]
    fn test_maximize_ignored_for_non_sidebar_window() {
        let temp_dir = tempdir().unwrap();
        let win = mock_window(99, true, false, 1, None);
        let mock = MockNiri::new(vec![win]);

        let mut ctx = Ctx {
            state: AppState::default(),
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        maximize(&mut ctx).expect("maximize should no-op");
        assert_eq!(ctx.state.maximized_window_id, None);
        assert!(ctx.socket.sent_actions.is_empty());
    }

    #[test]
    fn test_restore_uses_width_when_horizontal_sidebar() {
        let temp_dir = tempdir().unwrap();
        let win = mock_window(10, true, true, 1, Some((1.0, 2.0)));
        let mock = MockNiri::new(vec![win]);

        let mut state = AppState::default();
        state.windows.push(WindowState {
            id: 10,
            width: 1000,
            height: 800,
            is_floating: true,
            position: Some((1.0, 2.0)),
        });
        state.maximized_window_id = Some(10);

        let mut config = mock_config();
        config.interaction.position = SidebarPosition::Bottom;

        let mut ctx = Ctx {
            state,
            config,
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        maximize(&mut ctx).expect("maximize toggle-off failed");
        assert!(
            ctx.socket
                .sent_actions
                .iter()
                .any(|a| matches!(a, Action::SetWindowWidth { id: Some(10), .. }))
        );
    }
}
