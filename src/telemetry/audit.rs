use uuid::Uuid;

use crate::shared::validation::sanitize_for_logging;

#[derive(Clone)]
pub struct AuditLogger;

impl AuditLogger {
    pub fn new() -> Self {
        Self
    }

    pub fn log(&self, event: AuditEvent) {
        let actor_id = event.actor.id.map(|id| id.to_string());
        let actor_email = event
            .actor
            .email
            .as_ref()
            .map(|value| sanitize_for_logging(value))
            .unwrap_or_else(|| "-".to_string());
        let actor_role = event
            .actor
            .role
            .as_ref()
            .map(|value| sanitize_for_logging(value))
            .unwrap_or_else(|| "-".to_string());
        let target_id = event
            .target
            .id
            .as_ref()
            .map(|value| sanitize_for_logging(value))
            .unwrap_or_else(|| "-".to_string());
        let target_kind = sanitize_for_logging(&event.target.kind);
        let detail = event
            .detail
            .as_ref()
            .map(|value| sanitize_for_logging(value))
            .unwrap_or_else(|| "-".to_string());
        let ip = event
            .ip
            .as_ref()
            .map(|value| sanitize_for_logging(value))
            .unwrap_or_else(|| "-".to_string());
        let action = sanitize_for_logging(&event.action);
        let outcome = event.outcome.as_str();

        tracing::info!(
            target = "audit",
            action = %action,
            outcome = outcome,
            actor_id = actor_id.as_deref().unwrap_or("-"),
            actor_email = %actor_email,
            actor_role = %actor_role,
            target_kind = %target_kind,
            target_id = %target_id,
            ip = %ip,
            detail = %detail,
            "sensitive action recorded"
        );
    }
}

#[derive(Clone, Copy)]
pub enum AuditOutcome {
    Success,
    Failure,
}

impl AuditOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
        }
    }
}

#[derive(Clone, Default)]
pub struct AuditActor {
    pub id: Option<Uuid>,
    pub email: Option<String>,
    pub role: Option<String>,
}

#[derive(Clone)]
pub struct AuditTarget {
    pub kind: String,
    pub id: Option<String>,
}

impl AuditTarget {
    pub fn new(kind: impl Into<String>, id: Option<String>) -> Self {
        Self {
            kind: kind.into(),
            id,
        }
    }
}

#[derive(Clone)]
pub struct AuditEvent {
    pub action: String,
    pub actor: AuditActor,
    pub target: AuditTarget,
    pub outcome: AuditOutcome,
    pub detail: Option<String>,
    pub ip: Option<String>,
}

impl AuditEvent {
    pub fn success(
        action: impl Into<String>,
        actor: AuditActor,
        target: AuditTarget,
        detail: Option<String>,
        ip: Option<String>,
    ) -> Self {
        Self {
            action: action.into(),
            actor,
            target,
            outcome: AuditOutcome::Success,
            detail,
            ip,
        }
    }

    pub fn failure(
        action: impl Into<String>,
        actor: AuditActor,
        target: AuditTarget,
        detail: Option<String>,
        ip: Option<String>,
    ) -> Self {
        Self {
            action: action.into(),
            actor,
            target,
            outcome: AuditOutcome::Failure,
            detail,
            ip,
        }
    }
}
