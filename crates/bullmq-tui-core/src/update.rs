//! Update function - state transitions based on messages

use crate::effects::{Effect, UpdateResult};
use crate::message::Message;
use crate::model::{
    ConfirmAction, ConnectionStatus, JobDetailState, JobListState, Model, StatusMessage, View,
};

/// Process a message and update the model
///
/// This is the core of the Elm Architecture - a pure function that takes
/// the current state and a message, and returns the new state along with
/// any side effects that need to be executed.
pub fn update(model: &mut Model, msg: Message) -> UpdateResult {
    match msg {
        // === Navigation ===
        Message::Quit => {
            model.should_quit = true;
            UpdateResult::with_effect(Effect::Quit)
        }

        Message::Back => {
            match model.view {
                View::JobList => {
                    model.view = View::QueueList;
                    UpdateResult::changed()
                }
                View::JobDetail => {
                    model.view = View::JobList;
                    UpdateResult::changed()
                }
                View::Help | View::Search | View::Confirm => {
                    model.go_back();
                    UpdateResult::changed()
                }
                View::QueueList => {
                    // At root, quit
                    model.should_quit = true;
                    UpdateResult::with_effect(Effect::Quit)
                }
            }
        }

        Message::ShowHelp => {
            model.navigate(View::Help);
            UpdateResult::changed()
        }

        Message::HideOverlay => {
            if matches!(model.view, View::Help | View::Search | View::Confirm) {
                model.go_back();
            }
            model.confirm_action = None;
            UpdateResult::changed()
        }

        // === Queue List ===
        Message::RefreshQueues => {
            model.queue_list.loading = true;
            UpdateResult::with_effect(Effect::LoadQueues)
        }

        Message::QueuesLoaded(queues) => {
            model.queue_list.queues = queues.clone();
            model.queue_list.loading = false;
            model.connection = ConnectionStatus::Connected;

            // Load info for all queues
            let effects: Vec<Effect> = queues
                .into_iter()
                .map(Effect::LoadQueueInfo)
                .collect();

            UpdateResult::with_effects(effects)
        }

        Message::QueueInfoLoaded(name, info) => {
            model.queue_list.queue_info.insert(name, info);
            UpdateResult::changed()
        }

        Message::SelectNextQueue => {
            model.queue_list.select_next();
            UpdateResult::changed()
        }

        Message::SelectPrevQueue => {
            model.queue_list.select_prev();
            UpdateResult::changed()
        }

        Message::OpenQueue => {
            if let Some(queue_name) = model.queue_list.selected_queue() {
                let queue_name = queue_name.to_string();
                model.job_list = JobListState::new(queue_name.clone());
                model.view = View::JobList;

                UpdateResult::with_effects(vec![
                    Effect::LoadQueueCounts(queue_name.clone()),
                    Effect::LoadJobs {
                        queue: queue_name.clone(),
                        state: model.job_list.current_tab,
                        offset: 0,
                        count: model.job_list.page_size,
                    },
                    Effect::SubscribeEvents {
                        queue: queue_name,
                        last_id: None,
                    },
                ])
            } else {
                UpdateResult::none()
            }
        }

        // === Job List ===
        Message::RefreshJobs => {
            model.job_list.loading = true;
            let queue = model.job_list.queue_name.clone();
            let state = model.job_list.current_tab;
            let offset = model.job_list.offset;
            let count = model.job_list.page_size;

            UpdateResult::with_effects(vec![
                Effect::LoadQueueCounts(queue.clone()),
                Effect::LoadJobs {
                    queue,
                    state,
                    offset,
                    count,
                },
            ])
        }

        Message::JobsLoaded { jobs, total } => {
            model.job_list.jobs = jobs;
            model.job_list.total = total;
            model.job_list.loading = false;
            UpdateResult::changed()
        }

        Message::CountsUpdated(counts) => {
            model.job_list.counts = counts;
            UpdateResult::changed()
        }

        Message::SelectNextJob => {
            model.job_list.select_next();
            UpdateResult::changed()
        }

        Message::SelectPrevJob => {
            model.job_list.select_prev();
            UpdateResult::changed()
        }

        Message::NextTab => {
            model.job_list.next_tab();
            model.job_list.loading = true;

            let queue = model.job_list.queue_name.clone();
            let state = model.job_list.current_tab;

            UpdateResult::with_effect(Effect::LoadJobs {
                queue,
                state,
                offset: 0,
                count: model.job_list.page_size,
            })
        }

        Message::PrevTab => {
            model.job_list.prev_tab();
            model.job_list.loading = true;

            let queue = model.job_list.queue_name.clone();
            let state = model.job_list.current_tab;

            UpdateResult::with_effect(Effect::LoadJobs {
                queue,
                state,
                offset: 0,
                count: model.job_list.page_size,
            })
        }

        Message::SwitchTab(state) => {
            if model.job_list.current_tab != state {
                model.job_list.current_tab = state;
                model.job_list.selected = 0;
                model.job_list.offset = 0;
                model.job_list.loading = true;

                let queue = model.job_list.queue_name.clone();

                UpdateResult::with_effect(Effect::LoadJobs {
                    queue,
                    state,
                    offset: 0,
                    count: model.job_list.page_size,
                })
            } else {
                UpdateResult::none()
            }
        }

        Message::OpenJobDetail => {
            if let Some(job) = model.job_list.selected_job() {
                model.job_detail = JobDetailState {
                    job: Some(job.clone()),
                    scroll: 0,
                    loading: false,
                };
                model.view = View::JobDetail;
                UpdateResult::changed()
            } else {
                UpdateResult::none()
            }
        }

        Message::PageDown => {
            model.job_list.page_down();
            model.job_list.loading = true;

            let queue = model.job_list.queue_name.clone();
            let state = model.job_list.current_tab;
            let offset = model.job_list.offset;
            let count = model.job_list.page_size;

            UpdateResult::with_effect(Effect::LoadJobs {
                queue,
                state,
                offset,
                count,
            })
        }

        Message::PageUp => {
            model.job_list.page_up();
            model.job_list.loading = true;

            let queue = model.job_list.queue_name.clone();
            let state = model.job_list.current_tab;
            let offset = model.job_list.offset;
            let count = model.job_list.page_size;

            UpdateResult::with_effect(Effect::LoadJobs {
                queue,
                state,
                offset,
                count,
            })
        }

        // === Job Detail ===
        Message::JobDetailLoaded(job) => {
            model.job_detail.job = Some(job);
            model.job_detail.loading = false;
            UpdateResult::changed()
        }

        Message::ScrollDown => {
            model.job_detail.scroll_down();
            UpdateResult::changed()
        }

        Message::ScrollUp => {
            model.job_detail.scroll_up();
            UpdateResult::changed()
        }

        // === Job Actions ===
        Message::RetryJob => {
            if let Some(job) = model.job_list.selected_job() {
                model.confirm_action = Some(ConfirmAction::RetryJob {
                    queue: model.job_list.queue_name.clone(),
                    job_id: job.id.clone(),
                });
                model.navigate(View::Confirm);
                UpdateResult::changed()
            } else {
                UpdateResult::none()
            }
        }

        Message::RemoveJob => {
            if let Some(job) = model.job_list.selected_job() {
                model.confirm_action = Some(ConfirmAction::RemoveJob {
                    queue: model.job_list.queue_name.clone(),
                    job_id: job.id.clone(),
                });
                model.navigate(View::Confirm);
                UpdateResult::changed()
            } else {
                UpdateResult::none()
            }
        }

        Message::JobRetried(job_id) => {
            model.set_status(StatusMessage::info(format!("Job {} retried", job_id)));
            model.go_back();

            // Refresh job list
            UpdateResult::with_effect(Effect::LoadJobs {
                queue: model.job_list.queue_name.clone(),
                state: model.job_list.current_tab,
                offset: model.job_list.offset,
                count: model.job_list.page_size,
            })
        }

        Message::JobRemoved(job_id) => {
            model.set_status(StatusMessage::info(format!("Job {} removed", job_id)));
            model.go_back();

            // Refresh job list
            UpdateResult::with_effect(Effect::LoadJobs {
                queue: model.job_list.queue_name.clone(),
                state: model.job_list.current_tab,
                offset: model.job_list.offset,
                count: model.job_list.page_size,
            })
        }

        // === Queue Actions ===
        Message::PauseQueue => {
            model.confirm_action = Some(ConfirmAction::PauseQueue {
                queue: model.job_list.queue_name.clone(),
            });
            model.navigate(View::Confirm);
            UpdateResult::changed()
        }

        Message::ResumeQueue => {
            model.confirm_action = Some(ConfirmAction::ResumeQueue {
                queue: model.job_list.queue_name.clone(),
            });
            model.navigate(View::Confirm);
            UpdateResult::changed()
        }

        Message::QueuePaused(queue) => {
            model.set_status(StatusMessage::info(format!("Queue {} paused", queue)));
            model.go_back();
            UpdateResult::with_effect(Effect::LoadQueueInfo(queue))
        }

        Message::QueueResumed(queue) => {
            model.set_status(StatusMessage::info(format!("Queue {} resumed", queue)));
            model.go_back();
            UpdateResult::with_effect(Effect::LoadQueueInfo(queue))
        }

        // === Search ===
        Message::ToggleSearch => {
            if model.view == View::Search {
                model.search.active = false;
                model.go_back();
            } else {
                model.search.active = true;
                model.navigate(View::Search);
            }
            UpdateResult::changed()
        }

        Message::SearchInput(c) => {
            model.search.insert(c);
            UpdateResult::changed()
        }

        Message::SearchBackspace => {
            model.search.backspace();
            UpdateResult::changed()
        }

        Message::SearchDelete => {
            model.search.delete();
            UpdateResult::changed()
        }

        Message::SearchCursorLeft => {
            model.search.move_left();
            UpdateResult::changed()
        }

        Message::SearchCursorRight => {
            model.search.move_right();
            UpdateResult::changed()
        }

        Message::ClearSearch => {
            model.search.clear();
            UpdateResult::changed()
        }

        Message::ExecuteSearch => {
            // TODO: Implement search filtering
            model.search.active = false;
            model.go_back();
            UpdateResult::changed()
        }

        // === Confirmation ===
        Message::ShowConfirm => {
            model.navigate(View::Confirm);
            UpdateResult::changed()
        }

        Message::Confirm => {
            if let Some(action) = model.confirm_action.take() {
                let effect = match action {
                    ConfirmAction::RetryJob { queue, job_id } => Effect::RetryJob { queue, job_id },
                    ConfirmAction::RemoveJob { queue, job_id } => {
                        Effect::RemoveJob { queue, job_id }
                    }
                    ConfirmAction::PauseQueue { queue } => Effect::PauseQueue(queue),
                    ConfirmAction::ResumeQueue { queue } => Effect::ResumeQueue(queue),
                };
                model.go_back();
                UpdateResult::with_effect(effect)
            } else {
                model.go_back();
                UpdateResult::changed()
            }
        }

        Message::Cancel => {
            model.confirm_action = None;
            model.go_back();
            UpdateResult::changed()
        }

        // === Connection ===
        Message::Connected => {
            model.connection = ConnectionStatus::Connected;
            model.set_status(StatusMessage::info("Connected to Redis"));
            UpdateResult::with_effect(Effect::LoadQueues)
        }

        Message::Disconnected => {
            model.connection = ConnectionStatus::Disconnected;
            model.set_status(StatusMessage::error("Disconnected from Redis"));
            UpdateResult::changed()
        }

        Message::ConnectionError(err) => {
            model.connection = ConnectionStatus::Error(err.clone());
            model.set_status(StatusMessage::error(format!("Connection error: {}", err)));
            UpdateResult::changed()
        }

        // === Events ===
        Message::EventReceived(event) => {
            // Update last event ID
            let last_id = event.id.clone();
            model
                .last_event_ids
                .insert(event.queue_name.clone(), last_id.clone());

            // Trigger refresh if we're viewing this queue
            if model.view == View::JobList && model.job_list.queue_name == event.queue_name {
                let queue = event.queue_name.clone();
                UpdateResult::with_effects(vec![
                    Effect::LoadQueueCounts(queue.clone()),
                    Effect::LoadJobs {
                        queue: queue.clone(),
                        state: model.job_list.current_tab,
                        offset: model.job_list.offset,
                        count: model.job_list.page_size,
                    },
                    Effect::SubscribeEvents {
                        queue,
                        last_id: Some(last_id),
                    },
                ])
            } else {
                UpdateResult::changed()
            }
        }

        Message::EventsReceived(events) => {
            let mut last_id = None;
            for event in &events {
                model
                    .last_event_ids
                    .insert(event.queue_name.clone(), event.id.clone());
                last_id = Some(event.id.clone());
            }

            // Refresh jobs and counts if viewing affected queue
            if model.view == View::JobList {
                let queue = model.job_list.queue_name.clone();
                if events.iter().any(|e| e.queue_name == queue) {
                    return UpdateResult::with_effects(vec![
                        Effect::LoadQueueCounts(queue.clone()),
                        Effect::LoadJobs {
                            queue: queue.clone(),
                            state: model.job_list.current_tab,
                            offset: model.job_list.offset,
                            count: model.job_list.page_size,
                        },
                        // Continue subscribing for more events
                        Effect::SubscribeEvents {
                            queue,
                            last_id,
                        },
                    ]);
                }
            }

            UpdateResult::changed()
        }

        // === Errors ===
        Message::Error(err) => {
            model.set_status(StatusMessage::error(err));
            UpdateResult::changed()
        }

        // === Status ===
        Message::SetStatus(msg) => {
            model.set_status(StatusMessage::info(msg));
            UpdateResult::changed()
        }

        Message::ClearStatus => {
            model.clear_status();
            UpdateResult::changed()
        }

        // === Loading ===
        Message::SetLoading(loading) => {
            match model.view {
                View::QueueList => model.queue_list.loading = loading,
                View::JobList => model.job_list.loading = loading,
                View::JobDetail => model.job_detail.loading = loading,
                _ => {}
            }
            UpdateResult::changed()
        }

        // === Tick ===
        Message::Tick => {
            let mut effects = Vec::new();

            // Clear old status messages (after 5 seconds)
            if let Some(ts) = model.status.timestamp {
                let now = chrono::Utc::now().timestamp();
                if now - ts > 5 {
                    model.clear_status();
                }
            }

            // Auto-refresh when viewing job list
            if model.view == View::JobList && !model.job_list.loading {
                let queue = model.job_list.queue_name.clone();
                let last_id = model.last_event_ids.get(&queue).cloned();

                // Subscribe for events (non-blocking poll)
                effects.push(Effect::SubscribeEvents { queue, last_id });
            }

            if effects.is_empty() {
                UpdateResult::none()
            } else {
                UpdateResult::with_effects(effects)
            }
        }
    }
}
