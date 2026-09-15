use std::time::Duration;

use codex_protocol::ThreadId;
use codex_protocol::error::Result as CodexResult;

use super::AgentControl;

impl AgentControl {
    pub(crate) async fn wait_for_spine_spawn_turn_idle(
        &self,
        thread_id: ThreadId,
    ) -> CodexResult<()> {
        let state = self.upgrade()?;
        let thread = state.get_thread(thread_id).await?;
        loop {
            if thread.session.active_turn.lock().await.is_none() {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
}
