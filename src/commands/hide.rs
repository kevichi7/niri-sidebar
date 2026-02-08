use crate::Ctx;
use crate::commands::reorder;
use crate::state::save_state;
use anyhow::Result;

pub fn toggle_visibility(ctx: &mut Ctx) -> Result<()> {
    ctx.state.is_hidden = !ctx.state.is_hidden;
    save_state(&ctx.state)?;
    reorder(ctx)?;
    Ok(())
}
