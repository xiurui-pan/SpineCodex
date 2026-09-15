use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::Duration;

use codex_protocol::AgentPath;
use codex_protocol::ThreadId;
use futures::FutureExt;
use futures::StreamExt;
use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use spine_core::host::CollectWait;
use spine_core::host::SpawnReceipt;
use spine_core::host::SpawnResult;
use spine_core::host::SpawnTask;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use super::SpawnTransactionGuard;
use super::aborted_result;
use super::correct_intermediate_messages;
use super::is_spawn_terminal;
use super::quiesce_transaction_messages;
use super::result_from_status;
use super::result_status;
use super::spawn_progress_event;
use super::teardown_transaction_children_with_correction;
use super::wait_for_terminal;
use super::wave_receipt;
use crate::agent::AgentStatus;
use crate::session::MailboxSubmissionCancellation;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use codex_protocol::turn_input::TurnStartOptions;

pub(crate) struct SpawnWave {
    pub spawn_call_id: String,
    pub tasks: Vec<SpawnTask>,
    pub receipt: SpawnReceipt,
    pub pending_summaries: Vec<String>,
    pub complete: bool,
}

#[derive(Default)]
pub(crate) struct SpawnBatchRegistry {
    live: StdMutex<Option<Arc<LiveSpawnBatch>>>,
}

struct LiveSpawnBatch {
    spawn_call_id: String,
    tasks: Vec<SpawnTask>,
    parent_path: AgentPath,
    child_paths: Vec<AgentPath>,
    progress: Mutex<BatchProgress>,
    remaining_threads: Mutex<HashMap<usize, ThreadId>>,
    child_by_path: Mutex<HashMap<AgentPath, ThreadId>>,
    corrected_ids: Mutex<HashSet<String>>,
    notify: Notify,
    batch_cancel: CancellationToken,
    mailbox_cancellation: MailboxSubmissionCancellation,
    session: Arc<Session>,
    turn: Arc<TurnContext>,
    progress_thread_ids: Mutex<Vec<ThreadId>>,
    progress_statuses: Mutex<Vec<AgentStatus>>,
    guard: StdMutex<Option<SpawnTransactionGuard>>,
}

struct BatchProgress {
    remaining: HashSet<usize>,
    buffer: Vec<(usize, SpawnResult)>,
    outstanding: usize,
}

impl SpawnBatchRegistry {
    pub(crate) fn is_pending(&self) -> bool {
        self.live
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    pub(crate) fn reject_structural_control(&self) -> Result<(), String> {
        if self.is_pending() {
            return Err(
                "spine.open, spine.close, spine.next, and spine.spawn are unavailable while spawned branches are still running; call spine.collect first"
                    .to_string(),
            );
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn arm(
        &self,
        session: Arc<Session>,
        turn: Arc<TurnContext>,
        guard: SpawnTransactionGuard,
        spawn_call_id: String,
        tasks: Vec<SpawnTask>,
        parent_path: AgentPath,
        child_paths: Vec<AgentPath>,
        live: Vec<(usize, ThreadId, AgentPath)>,
        mailbox_cancellation: MailboxSubmissionCancellation,
        progress_thread_ids: Vec<ThreadId>,
        progress_statuses: Vec<AgentStatus>,
        batch_cancel: CancellationToken,
    ) -> Result<(), String> {
        let mut slot = self
            .live
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot.is_some() {
            return Err("spine.spawn cannot start while another transaction is active".to_string());
        }
        let remaining = live.iter().map(|(ordinal, _, _)| *ordinal).collect();
        let remaining_threads = live
            .iter()
            .map(|(ordinal, thread_id, _)| (*ordinal, *thread_id))
            .collect::<HashMap<_, _>>();
        let child_by_path = live
            .iter()
            .map(|(_, thread_id, path)| (path.clone(), *thread_id))
            .collect();
        let outstanding = live.len();
        let waits = live
            .iter()
            .map(|(ordinal, thread_id, path)| (*ordinal, *thread_id, path.clone()))
            .collect::<Vec<_>>();
        let batch = Arc::new(LiveSpawnBatch {
            spawn_call_id,
            tasks,
            parent_path,
            child_paths,
            progress: Mutex::new(BatchProgress {
                remaining,
                buffer: Vec::new(),
                outstanding,
            }),
            remaining_threads: Mutex::new(remaining_threads),
            child_by_path: Mutex::new(child_by_path),
            corrected_ids: Mutex::new(HashSet::new()),
            notify: Notify::new(),
            batch_cancel: batch_cancel.clone(),
            mailbox_cancellation,
            session: Arc::clone(&session),
            turn,
            progress_thread_ids: Mutex::new(progress_thread_ids),
            progress_statuses: Mutex::new(progress_statuses),
            guard: StdMutex::new(Some(guard)),
        });
        *slot = Some(Arc::clone(&batch));
        drop(slot);
        tokio::spawn(run_waiters(batch, waits, batch_cancel));
        Ok(())
    }

    pub(crate) async fn collect(
        &self,
        wait: CollectWait,
        tool_cancel: &CancellationToken,
        abort_on_tool_cancel: bool,
    ) -> Result<SpawnWave, String> {
        let live = self
            .live
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or_else(|| {
                "no pending spine.spawn branches; spine.collect is only valid while a spawn batch is still running"
                    .to_string()
            })?;
        loop {
            if live.batch_cancel.is_cancelled() {
                live.abort().await;
                self.clear();
                return Err("spine.spawn was cancelled before child completion".to_string());
            }
            let notified = live.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if let Some(wave) = self.try_harvest(&live, wait).await? {
                return Ok(wave);
            }
            tokio::select! {
                () = notified => {}
                () = tool_cancel.cancelled() => {
                    if abort_on_tool_cancel {
                        live.abort().await;
                        self.clear();
                    }
                    return Err(if abort_on_tool_cancel {
                        "spine.spawn was cancelled before child completion".to_string()
                    } else {
                        "spine.collect was cancelled before child completion".to_string()
                    });
                }
                () = live.batch_cancel.cancelled() => {
                    live.abort().await;
                    self.clear();
                    return Err("spine.spawn was cancelled before child completion".to_string());
                }
            }
        }
    }

    async fn try_harvest(
        &self,
        live: &Arc<LiveSpawnBatch>,
        wait: CollectWait,
    ) -> Result<Option<SpawnWave>, String> {
        let mut progress = live.progress.lock().await;
        if progress.buffer.is_empty() {
            if progress.outstanding == 0 {
                return Err("spine.spawn batch finished without buffered results".to_string());
            }
            return Ok(None);
        }
        if matches!(wait, CollectWait::All) && progress.outstanding > 0 {
            return Ok(None);
        }
        let completed = std::mem::take(&mut progress.buffer);
        let wave = match wave_receipt(&live.tasks, completed.clone()) {
            Ok(wave) => wave,
            Err(error) => {
                progress.buffer = completed;
                return Err(error);
            }
        };
        let (tasks, receipt) = wave;
        let pending_summaries = live
            .tasks
            .iter()
            .enumerate()
            .filter(|(ordinal, _)| progress.remaining.contains(ordinal))
            .map(|(_, task)| task.summary.clone())
            .collect::<Vec<_>>();
        let complete = progress.remaining.is_empty() && progress.outstanding == 0;
        drop(progress);
        if complete {
            self.clear();
        }
        Ok(Some(SpawnWave {
            spawn_call_id: live.spawn_call_id.clone(),
            tasks,
            receipt,
            pending_summaries,
            complete,
        }))
    }

    fn clear(&self) {
        let live = self
            .live
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(live) = live {
            live.release_lifecycle();
        }
    }
}

impl LiveSpawnBatch {
    fn release_lifecycle(&self) {
        self.guard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }

    async fn abort(&self) {
        self.batch_cancel.cancel();
        self.mailbox_cancellation.activate();
        let remaining_threads = {
            let mut remaining_threads = self.remaining_threads.lock().await;
            std::mem::take(&mut *remaining_threads)
        };
        let thread_ids = remaining_threads.values().copied().collect::<Vec<_>>();
        let paths = remaining_threads
            .keys()
            .map(|ordinal| self.child_paths[*ordinal].clone())
            .collect::<Vec<_>>();
        let child_by_path = self.child_by_path.lock().await.clone();
        let mut corrected_ids = self.corrected_ids.lock().await;
        let _ = teardown_transaction_children_with_correction(
            &self.session,
            &self.parent_path,
            &thread_ids,
            &paths,
            &child_by_path,
            &mut corrected_ids,
        )
        .await;
        quiesce_transaction_messages(
            &self.session,
            &self.parent_path,
            &paths,
            &child_by_path,
            &mut corrected_ids,
        )
        .await;
        self.release_lifecycle();
        self.notify.notify_waiters();
    }
}

fn run_waiters(
    batch: Arc<LiveSpawnBatch>,
    waits: Vec<(usize, ThreadId, AgentPath)>,
    batch_cancel: CancellationToken,
) -> impl std::future::Future<Output = ()> {
    async move {
        let mut pending = waits
            .into_iter()
            .map(|(ordinal, thread_id, path)| {
                child_wait(Arc::clone(&batch), ordinal, thread_id, path)
            })
            .collect::<FuturesUnordered<_>>();
        let mut interval = tokio::time::interval(Duration::from_millis(25));
        loop {
            tokio::select! {
                item = pending.next() => {
                    let Some((ordinal, result)) = item else {
                        break;
                    };
                    complete_child(&batch, ordinal, result).await;
                }
                () = batch_cancel.cancelled() => break,
                _ = interval.tick() => {
                    let child_by_path = batch.child_by_path.lock().await.clone();
                    let mut corrected_ids = batch.corrected_ids.lock().await;
                    correct_intermediate_messages(
                        &batch.session,
                        &batch.parent_path,
                        &batch.child_paths,
                        &child_by_path,
                        &mut corrected_ids,
                    )
                    .await;
                }
            }
        }
        if batch_cancel.is_cancelled() {
            batch.abort().await;
            batch.session.spine_spawn_batch.clear();
        } else {
            batch.notify.notify_waiters();
        }
    }
}

fn child_wait(
    batch: Arc<LiveSpawnBatch>,
    ordinal: usize,
    thread_id: ThreadId,
    path: AgentPath,
) -> BoxFuture<'static, (usize, SpawnResult)> {
    async move {
        let start_options = TurnStartOptions {
            parent_turn_id: Some(batch.turn.sub_id.clone()),
            root_turn_id: batch.turn.turn_metadata_state.root_turn_id(),
            cyber_access_program: batch.turn.cyber_access_program,
            ..Default::default()
        };
        let mut status = wait_for_terminal(
            &batch.session.services.agent_control,
            &batch.parent_path,
            &path,
            batch.session.thread_id,
            start_options,
            thread_id,
        )
        .await;
        if batch
            .session
            .services
            .agent_control
            .wait_for_spine_spawn_turn_idle(thread_id)
            .await
            .is_ok()
        {
            status = batch
                .session
                .services
                .agent_control
                .get_status(thread_id)
                .await;
        }
        if !is_spawn_terminal(&status) && batch.batch_cancel.is_cancelled() {
            return (ordinal, aborted_result(ordinal, Some(thread_id)));
        }
        (ordinal, result_from_status(ordinal, thread_id, status))
    }
    .boxed()
}

async fn complete_child(batch: &LiveSpawnBatch, ordinal: usize, result: SpawnResult) {
    {
        let mut statuses = batch.progress_statuses.lock().await;
        statuses[ordinal] = result_status(&result);
        let thread_ids = batch.progress_thread_ids.lock().await;
        let event = spawn_progress_event(
            &batch.spawn_call_id,
            &batch.tasks,
            &thread_ids,
            &batch.child_paths,
            &statuses,
        );
        drop(thread_ids);
        drop(statuses);
        batch
            .session
            .emit_spine_spawn_progress(batch.turn.as_ref(), event)
            .await;
    }
    let thread_id = {
        let mut remaining_threads = batch.remaining_threads.lock().await;
        remaining_threads.remove(&ordinal)
    };
    if let Some(thread_id) = thread_id {
        let child_by_path = batch.child_by_path.lock().await.clone();
        let mut corrected_ids = batch.corrected_ids.lock().await;
        let path = [batch.child_paths[ordinal].clone()];
        let _ = teardown_transaction_children_with_correction(
            &batch.session,
            &batch.parent_path,
            &[thread_id],
            &path,
            &child_by_path,
            &mut corrected_ids,
        )
        .await;
        quiesce_transaction_messages(
            &batch.session,
            &batch.parent_path,
            &path,
            &child_by_path,
            &mut corrected_ids,
        )
        .await;
    }
    {
        let mut progress = batch.progress.lock().await;
        progress.buffer.push((ordinal, result));
        progress.remaining.remove(&ordinal);
        progress.outstanding = progress.outstanding.saturating_sub(1);
    }
    batch.notify.notify_waiters();
}
