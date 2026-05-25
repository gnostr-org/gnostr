use std::sync::{Mutex, OnceLock};

use gnostr_chat::{global_rt, ChatNotification, ChatSession};
use tokio::sync::broadcast::{error::TryRecvError, Receiver};

struct ChatHandle {
    topic: String,
    status: String,
    session: ChatSession,
    updates: Receiver<ChatNotification>,
}

static CHAT: OnceLock<Mutex<Option<ChatHandle>>> = OnceLock::new();
static CHAT_LOGS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn chat_slot() -> &'static Mutex<Option<ChatHandle>> {
    CHAT.get_or_init(|| Mutex::new(None))
}

fn chat_logs_slot() -> &'static Mutex<Vec<String>> {
    CHAT_LOGS.get_or_init(|| Mutex::new(Vec::new()))
}

fn normalize_topic(topic: &str) -> String {
    let trimmed = topic.trim();
    if trimmed.is_empty() {
        "gnostr-dev".to_string()
    } else {
        trimmed.to_string()
    }
}

fn push_chat_log(line: impl Into<String>) {
    let line = line.into();
    let mut logs = chat_logs_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    logs.push(line);
    if logs.len() > 500 {
        let drain_to = logs.len().saturating_sub(500);
        logs.drain(0..drain_to);
    }
}

fn clear_chat_logs() {
    chat_logs_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
}

fn format_notification(notification: &ChatNotification) -> String {
    match notification {
        ChatNotification::ChatMessage(msg) => msg.to_string(),
        ChatNotification::Info(text) => text.clone(),
        ChatNotification::Error(text) => format!("[error] {text}"),
        ChatNotification::Connected { peer_id, endpoint } => {
            format!("connected peer={peer_id} endpoint={endpoint}")
        }
    }
}

fn drain_updates(handle: &mut ChatHandle) {
    loop {
        match handle.updates.try_recv() {
            Ok(notification) => {
                push_chat_log(format_notification(&notification));
                match notification {
                    ChatNotification::Connected { peer_id, .. } => {
                        handle.status = format!("connected chat topic={} peer={peer_id}", handle.topic);
                    }
                    ChatNotification::Error(text) => {
                        handle.status = format!("chat error topic={}: {text}", handle.topic);
                    }
                    ChatNotification::ChatMessage(_) | ChatNotification::Info(_) => {}
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Lagged(skipped)) => {
                let status = format!("chat updates lagged topic={} skipped={skipped}", handle.topic);
                push_chat_log(status.clone());
                handle.status = status;
            }
            Err(TryRecvError::Closed) => {
                let status = format!("chat updates closed topic={}", handle.topic);
                push_chat_log(status.clone());
                handle.status = status;
                break;
            }
        }
    }
}

pub fn chat_current_topic() -> String {
    let mut guard = chat_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(handle) = guard.as_mut() {
        drain_updates(handle);
        handle.topic.clone()
    } else {
        "gnostr-dev".to_string()
    }
}

pub fn chat_start(topic: String) -> String {
    let topic = normalize_topic(&topic);
    let mut guard = chat_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(mut existing) = guard.take() {
        drain_updates(&mut existing);
        if existing.topic == topic {
            let status = existing.status.clone();
            *guard = Some(existing);
            return status;
        }
        push_chat_log(format!("stopped chat topic={}", existing.topic));
    } else {
        clear_chat_logs();
    }

    let initial_status = format!("starting chat topic={topic}");
    push_chat_log(initial_status.clone());

    match global_rt().block_on(ChatSession::connect(topic.clone())) {
        Ok(session) => {
            let mut handle = ChatHandle {
                topic,
                status: initial_status,
                updates: session.subscribe(),
                session,
            };
            drain_updates(&mut handle);
            let status = handle.status.clone();
            *guard = Some(handle);
            status
        }
        Err(error) => {
            let status = format!("chat failed to start topic={topic}: {error}");
            push_chat_log(status.clone());
            status
        }
    }
}

pub fn chat_stop() -> String {
    let state = {
        let mut guard = chat_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.take()
    };

    let Some(mut state) = state else {
        return "chat not running".to_string();
    };

    drain_updates(&mut state);
    let status = format!("stopped chat topic={}", state.topic);
    push_chat_log(status.clone());
    status
}

pub fn chat_status() -> String {
    let mut guard = chat_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(handle) = guard.as_mut() else {
        return "chat not running".to_string();
    };

    drain_updates(handle);
    handle.status.clone()
}

pub fn chat_logs() -> String {
    {
        let mut guard = chat_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(handle) = guard.as_mut() {
            drain_updates(handle);
        }
    }

    chat_logs_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .join("\n")
}

pub fn chat_send(text: String) -> String {
    let mut guard = chat_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(handle) = guard.as_mut() else {
        return "chat not running".to_string();
    };

    drain_updates(handle);

    let message = text.trim().to_string();
    if message.is_empty() {
        let status = format!("chat message is empty topic={}", handle.topic);
        push_chat_log(status.clone());
        handle.status = status.clone();
        return status;
    }

    match global_rt().block_on(handle.session.send_text(message.clone())) {
        Ok(()) => {
            push_chat_log(format!("me: {message}"));
            let status = format!("sent chat message topic={}", handle.topic);
            handle.status = status.clone();
            status
        }
        Err(error) => {
            let status = format!("chat send failed topic={}: {error}", handle.topic);
            push_chat_log(status.clone());
            handle.status = status.clone();
            status
        }
    }
}
