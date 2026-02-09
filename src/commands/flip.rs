use crate::Ctx;
use crate::commands::reorder;
use crate::niri::NiriClient;
use crate::state::save_state;
use anyhow::Result;

pub fn toggle_flip<C: NiriClient>(ctx: &mut Ctx<C>) -> Result<()> {
    ctx.state.is_flipped = !ctx.state.is_flipped;
    save_state(&ctx.state, &ctx.cache_dir)?;
    reorder(ctx)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use crate::test_utils::{MockNiri, mock_config, mock_window};
    use niri_ipc::{Action, PositionChange};
    use tempfile::tempdir;

    #[test]
    fn test_toggle_flip() {
        let temp_dir = tempdir().unwrap();

        let w1 = mock_window(1, false, true, 1);
        let w2 = mock_window(2, true, true, 1);
        let mock = MockNiri::new(vec![w1, w2]);

        let mut state = AppState::default();
        state.windows.push((1, 300, 500)); // Index 0
        state.windows.push((2, 300, 500)); // Index 1
        state.is_flipped = false;

        let mut ctx = Ctx {
            state,
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        toggle_flip(&mut ctx).expect("Toggle flip failed");
        assert!(ctx.state.is_flipped);

        // Check reorder happened:
        // Normally (Unflipped): Index 0 is bottom, Index 1 is top.
        // Flipped: Index 1 (Window 2) should be at the bottom Y.

        // Base Y (Bottom slot) = 1080 - 200 - 50 = 830.
        let actions = &ctx.socket.sent_actions;
        dbg!(actions);
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::MoveFloatingWindow {
                id: Some(2),
                y: PositionChange::SetFixed(830.0),
                ..
            }
        )));
    }
}
