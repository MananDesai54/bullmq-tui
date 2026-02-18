//! BullMQ TUI - Terminal Application
//!
//! A terminal-based monitoring tool for BullMQ queues.

mod event_handler;

use anyhow::Result;
use bullmq_core::{BullMQClient, BullMQEventClient, RedisClient};
use bullmq_tui_core::{update, Effect, Message, Model};
use clap::Parser;
use crossterm::{
    event::{self, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures_util::StreamExt;
use ratatui::prelude::*;
use std::io::stdout;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// BullMQ Monitor TUI
#[derive(Parser, Debug)]
#[command(name = "bullmq-tui")]
#[command(about = "A terminal UI for monitoring BullMQ queues")]
#[command(version)]
struct Args {
    /// Redis connection URL
    #[arg(short, long, default_value = "redis://localhost:6379")]
    redis: String,

    /// Refresh interval in seconds
    #[arg(short, long, default_value = "5")]
    interval: u64,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    let log_level = if args.debug { "debug" } else { "warn" };
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_target(false)
        .init();

    info!("Starting BullMQ TUI");
    info!("Connecting to Redis at {}", args.redis);

    // Run the TUI
    let result = run_app(args).await;

    // Ensure terminal is restored even on error
    if let Err(e) = &result {
        error!("Application error: {}", e);
    }

    result
}

async fn run_app(args: Args) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create channels for async communication
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Initialize model
    let mut model = Model::new(&args.redis);

    // Connect to Redis
    let client = match RedisClient::connect(&args.redis).await {
        Ok(client) => {
            tx.send(Message::Connected)?;
            Arc::new(client)
        }
        Err(e) => {
            tx.send(Message::ConnectionError(e.to_string()))?;
            // Continue anyway, will show error in UI
            match RedisClient::connect(&args.redis).await {
                Ok(c) => Arc::new(c),
                Err(_) => {
                    // Can't connect at all, exit after showing error
                    terminal.draw(|f| bullmq_tui_core::render(&model, f))?;
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    cleanup_terminal()?;
                    return Err(anyhow::anyhow!("Failed to connect to Redis: {}", e));
                }
            }
        }
    };

    // Spawn event reader task
    let tx_events = tx.clone();
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        loop {
            if let Some(Ok(_event)) = reader.next().await {
                // We'll handle the event in the main loop
                // For now, just send a Tick to trigger event polling
                let _ = tx_events.send(Message::Tick);
            }
        }
    });

    // Spawn tick timer for periodic updates
    let tx_tick = tx.clone();
    let interval = args.interval;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval));
        loop {
            interval.tick().await;
            if tx_tick.send(Message::Tick).is_err() {
                break;
            }
        }
    });

    // Initial load
    tx.send(Message::RefreshQueues)?;

    // Main event loop
    loop {
        // Draw the UI
        terminal.draw(|f| bullmq_tui_core::render(&model, f))?;

        // Handle input events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Ok(event) = event::read() {
                if let Some(msg) = event_handler::handle_event(event, &model) {
                    let result = update(&mut model, msg);

                    // Execute effects
                    for effect in result.effects {
                        execute_effect(&client, &tx, effect).await;
                    }
                }
            }
        }

        // Process messages from channel
        while let Ok(msg) = rx.try_recv() {
            let result = update(&mut model, msg);

            // Execute effects
            for effect in result.effects {
                execute_effect(&client, &tx, effect).await;
            }
        }

        // Check if we should quit
        if model.should_quit {
            break;
        }
    }

    cleanup_terminal()?;
    Ok(())
}

/// Execute a side effect
async fn execute_effect(
    client: &Arc<RedisClient>,
    tx: &mpsc::UnboundedSender<Message>,
    effect: Effect,
) {
    match effect {
        Effect::None => {}

        Effect::Batch(effects) => {
            for e in effects {
                Box::pin(execute_effect(client, tx, e)).await;
            }
        }

        Effect::LoadQueues => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match client.discover_queues().await {
                    Ok(queues) => {
                        let _ = tx.send(Message::QueuesLoaded(queues));
                    }
                    Err(e) => {
                        let _ = tx.send(Message::Error(format!("Failed to load queues: {}", e)));
                    }
                }
            });
        }

        Effect::LoadQueueInfo(queue_name) => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match client.get_queue_info(&queue_name).await {
                    Ok(info) => {
                        let _ = tx.send(Message::QueueInfoLoaded(queue_name, info));
                    }
                    Err(e) => {
                        debug!("Failed to load queue info for {}: {}", queue_name, e);
                    }
                }
            });
        }

        Effect::LoadJobs {
            queue,
            state,
            offset,
            count,
        } => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match client.get_jobs(&queue, state, offset, count).await {
                    Ok(jobs) => {
                        // Also get total count
                        let total = client
                            .get_queue_counts(&queue)
                            .await
                            .map(|c| c.get(state))
                            .unwrap_or(jobs.len() as u64);

                        let _ = tx.send(Message::JobsLoaded { jobs, total });
                    }
                    Err(e) => {
                        let _ = tx.send(Message::Error(format!("Failed to load jobs: {}", e)));
                    }
                }
            });
        }

        Effect::LoadJobDetail { queue, job_id } => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match client.get_job(&queue, &job_id).await {
                    Ok(Some(job)) => {
                        let _ = tx.send(Message::JobDetailLoaded(job));
                    }
                    Ok(None) => {
                        let _ = tx.send(Message::Error(format!("Job {} not found", job_id)));
                    }
                    Err(e) => {
                        let _ = tx.send(Message::Error(format!("Failed to load job: {}", e)));
                    }
                }
            });
        }

        Effect::LoadQueueCounts(queue_name) => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match client.get_queue_counts(&queue_name).await {
                    Ok(counts) => {
                        let _ = tx.send(Message::CountsUpdated(counts));
                    }
                    Err(e) => {
                        debug!("Failed to load queue counts: {}", e);
                    }
                }
            });
        }

        Effect::RetryJob { queue, job_id } => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match client.retry_job(&queue, &job_id).await {
                    Ok(()) => {
                        let _ = tx.send(Message::JobRetried(job_id));
                    }
                    Err(e) => {
                        let _ = tx.send(Message::Error(format!("Failed to retry job: {}", e)));
                    }
                }
            });
        }

        Effect::RemoveJob { queue, job_id } => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match client.remove_job(&queue, &job_id).await {
                    Ok(()) => {
                        let _ = tx.send(Message::JobRemoved(job_id));
                    }
                    Err(e) => {
                        let _ = tx.send(Message::Error(format!("Failed to remove job: {}", e)));
                    }
                }
            });
        }

        Effect::PauseQueue(queue_name) => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match client.pause_queue(&queue_name).await {
                    Ok(()) => {
                        let _ = tx.send(Message::QueuePaused(queue_name));
                    }
                    Err(e) => {
                        let _ = tx.send(Message::Error(format!("Failed to pause queue: {}", e)));
                    }
                }
            });
        }

        Effect::ResumeQueue(queue_name) => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match client.resume_queue(&queue_name).await {
                    Ok(()) => {
                        let _ = tx.send(Message::QueueResumed(queue_name));
                    }
                    Err(e) => {
                        let _ = tx.send(Message::Error(format!("Failed to resume queue: {}", e)));
                    }
                }
            });
        }

        Effect::SubscribeEvents { queue, last_id } => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match client
                    .subscribe_events(&queue, last_id.as_deref())
                    .await
                {
                    Ok(events) if !events.is_empty() => {
                        let _ = tx.send(Message::EventsReceived(events));
                    }
                    Ok(_) => {} // No events
                    Err(e) => {
                        debug!("Event subscription error: {}", e);
                    }
                }
            });
        }

        Effect::Connect(_url) => {
            // Already connected in main
            let _ = tx.send(Message::Connected);
        }

        Effect::CheckConnection => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                if client.is_connected().await {
                    let _ = tx.send(Message::Connected);
                } else {
                    let _ = tx.send(Message::Disconnected);
                }
            });
        }

        Effect::Quit => {
            // Handled in main loop
        }
    }
}

/// Clean up terminal state
fn cleanup_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
