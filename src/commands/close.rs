use crate::Ctx;
use crate::commands::reorder;
use crate::niri::NiriClient;
use crate::state::save_state;
use anyhow::{Context, Result};
use niri_ipc::Action;

pub fn close<C: NiriClient>(ctx: &mut Ctx<C>) -> Result<()> {
    let windows = ctx.socket.get_windows()?;
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
        save_state(&ctx.state, &ctx.cache_dir)?;
    }

    let _ = ctx.socket.send_action(Action::CloseWindow {
        id: Some(focused.id),
    });
    reorder(ctx)?;

    Ok(())
}

#[cfg(test)]
mod tests_close {
    use super::*;
    use crate::state::AppState;
    use crate::test_utils::{MockNiri, mock_config, mock_window};
    use niri_ipc::Action;
    use tempfile::tempdir;

    #[test]
    fn test_close_sidebar_window() {
        let temp_dir = tempdir().unwrap();
        let win = mock_window(10, true, true, 1);
        let mock = MockNiri::new(vec![win]);

        let mut state = AppState::default();
        state.windows.push((10, 100, 100));

        let mut ctx = Ctx {
            state,
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        close(&mut ctx).expect("Close failed");

        assert!(
            ctx.state.windows.is_empty(),
            "Window was not removed from state"
        );

        assert!(
            ctx.socket
                .sent_actions
                .iter()
                .any(|a| matches!(a, Action::CloseWindow { id: Some(10) }))
        );
    }

    #[test]
    fn test_close_untracked_window() {
        let temp_dir = tempdir().unwrap();
        // Focused window (ID 99) is NOT in the sidebar
        let w1 = mock_window(99, true, false, 1);
        let w2 = mock_window(10, false, true, 1);
        let mock = MockNiri::new(vec![w1, w2]);

        let mut state = AppState::default();
        state.windows.push((10, 100, 100));

        let mut ctx = Ctx {
            state,
            config: Default::default(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        close(&mut ctx).expect("Close failed");

        assert_eq!(ctx.state.windows.len(), 1);
        assert_eq!(ctx.state.windows[0].0, 10);

        // CloseWindow action still sent
        assert!(
            ctx.socket
                .sent_actions
                .iter()
                .any(|a| matches!(a, Action::CloseWindow { id: Some(99) }))
        );
    }
}
