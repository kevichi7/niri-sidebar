use crate::Ctx;
use crate::commands::maximize::restore_sidebar_window_sizes;
use crate::commands::reorder;
use crate::niri::NiriClient;
use crate::state::save_state;
use anyhow::Result;

pub fn toggle_maximize_focus_mode<C: NiriClient>(ctx: &mut Ctx<C>) -> Result<()> {
    ctx.state.maximize_focus_mode = !ctx.state.maximize_focus_mode;

    if !ctx.state.maximize_focus_mode {
        if ctx.state.maximized_window_id.is_some() {
            ctx.state.maximized_window_id = None;
            restore_sidebar_window_sizes(ctx)?;
        }
    } else if ctx.state.maximize_focus_mode
        && let Ok(window) = ctx.socket.get_active_window()
        && ctx.state.windows.iter().any(|w| w.id == window.id)
    {
        ctx.state.maximized_window_id = Some(window.id);
    }

    save_state(&ctx.state, &ctx.cache_dir)?;
    reorder(ctx)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppState, WindowState};
    use crate::test_utils::{MockNiri, mock_config, mock_window};
    use niri_ipc::Action;
    use tempfile::tempdir;

    #[test]
    fn test_disable_mode_clears_maximized_and_restores_sizes() {
        let temp_dir = tempdir().unwrap();
        let focused = mock_window(10, true, true, 1, Some((1.0, 2.0)));
        let other = mock_window(20, false, true, 1, Some((1.0, 2.0)));
        let mock = MockNiri::new(vec![focused, other]);

        let mut state = AppState {
            maximize_focus_mode: true,
            maximized_window_id: Some(10),
            ..Default::default()
        };
        state.windows.push(WindowState {
            id: 10,
            width: 1000,
            height: 800,
            is_floating: true,
            position: Some((1.0, 2.0)),
        });
        state.windows.push(WindowState {
            id: 20,
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

        toggle_maximize_focus_mode(&mut ctx).expect("toggle should succeed");

        assert!(!ctx.state.maximize_focus_mode);
        assert_eq!(ctx.state.maximized_window_id, None);
        assert!(
            ctx.socket
                .sent_actions
                .iter()
                .any(|a| matches!(a, Action::SetWindowHeight { id: Some(10), .. }))
        );
        assert!(
            ctx.socket
                .sent_actions
                .iter()
                .any(|a| matches!(a, Action::SetWindowHeight { id: Some(20), .. }))
        );
    }
}
