use super::scoped_state::{CloseGuard, EventSink};
use anyhow::{bail, Context as _};
use core::time::Duration;
use nostr_sdk::pool::RelayNotification;
use nostr_sdk::{ClientMessage, Event, Filter, Relay, RelayMessage, RelayStatus, SubscriptionId};
use tokio::sync::broadcast::error::RecvError;
use tokio::time::{timeout_at, Instant};

mod auth;
mod progress;
mod validation;
pub(super) use progress::is_local_progress_backpressure;
pub(super) use validation::event_limit;

pub(super) struct QueryRequest {
    pub(super) relay: Relay,
    pub(super) filter: Filter,
    pub(super) timeout: Duration,
    pub(super) readiness_timeout: Duration,
    pub(super) sink: EventSink,
}

pub(super) struct PreparedQuery {
    pub(super) query: ScopedQuery,
    pub(super) cancellation_guard: CloseGuard,
}

impl PreparedQuery {
    pub(super) fn new(request: QueryRequest) -> Self {
        let id = SubscriptionId::generate();
        let notifications = request.relay.notifications();
        let close = CloseGuard::new(request.relay.clone(), id.clone());
        let query = ScopedQuery::new(request, id, notifications, close.clone());
        Self {
            query,
            cancellation_guard: close,
        }
    }
}

pub(super) struct QueryCompletion {
    pub(super) authoritative: bool,
}

pub(super) struct ScopedQuery {
    relay: Relay,
    filter: Filter,
    validation: validation::ValidationState,
    timeout: Duration,
    readiness_timeout: Duration,
    sink: EventSink,
    id: SubscriptionId,
    notifications: tokio::sync::broadcast::Receiver<RelayNotification>,
    _close: CloseGuard,
    lagged: bool,
    authenticated: bool,
    authentication_failed: bool,
    awaiting_auth: bool,
    auth_retried: bool,
    delivering_progress: bool,
}

impl ScopedQuery {
    fn new(
        request: QueryRequest,
        id: SubscriptionId,
        notifications: tokio::sync::broadcast::Receiver<RelayNotification>,
        close: CloseGuard,
    ) -> Self {
        let validation = validation::ValidationState::new(&request.filter);
        Self {
            relay: request.relay,
            filter: request.filter,
            validation,
            timeout: request.timeout,
            readiness_timeout: request.readiness_timeout,
            sink: request.sink,
            id,
            notifications,
            _close: close,
            lagged: false,
            authenticated: false,
            authentication_failed: false,
            awaiting_auth: false,
            auth_retried: false,
            delivering_progress: false,
        }
    }

    pub(super) async fn run(mut self) -> anyhow::Result<QueryCompletion> {
        let connection_deadline = Instant::now() + self.readiness_timeout;
        self.await_connected(connection_deadline).await?;
        let deadline = Instant::now() + self.timeout;
        self.send_request()?;
        loop {
            match timeout_at(deadline, self.notifications.recv()).await {
                Err(_) => bail!("query timed out"),
                Ok(Err(RecvError::Closed)) => bail!("notification channel closed"),
                Ok(Err(RecvError::Lagged(_))) => self.lagged = true,
                Ok(Ok(notification)) => {
                    if let Some(completion) = self.handle_before(deadline, notification).await? {
                        return Ok(completion);
                    }
                }
            }
        }
    }

    async fn await_connected(&mut self, deadline: Instant) -> anyhow::Result<()> {
        if self.relay.status() == RelayStatus::Connected {
            return Ok(());
        }
        self.relay.connect(None).await;
        loop {
            if self.relay.status() == RelayStatus::Connected {
                return Ok(());
            }
            match timeout_at(deadline, self.notifications.recv()).await {
                Err(_) => bail!("relay did not connect before query deadline"),
                Ok(Err(RecvError::Closed)) => bail!("notification channel closed"),
                Ok(Ok(RelayNotification::Shutdown)) => bail!("relay shut down before query"),
                Ok(_) => {}
            }
        }
    }

    fn send_request(&self) -> anyhow::Result<()> {
        let request = ClientMessage::req(self.id.clone(), vec![self.filter.clone()]);
        self.relay.send_msg(request).context("could not send REQ")
    }

    fn retry_request(&mut self) -> anyhow::Result<()> {
        let id = SubscriptionId::generate();
        self._close = CloseGuard::new(self.relay.clone(), id.clone());
        self.id = id;
        self.send_request()
    }

    async fn handle(
        &mut self,
        notification: RelayNotification,
    ) -> anyhow::Result<Option<QueryCompletion>> {
        match notification {
            RelayNotification::Message { message } => self.handle_message(message).await,
            RelayNotification::Authenticated => self.handle_authenticated(),
            RelayNotification::AuthenticationFailed => self.handle_authentication_failed(),
            RelayNotification::RelayStatus {
                status: RelayStatus::Disconnected | RelayStatus::Terminated,
            } => {
                bail!("relay disconnected during query")
            }
            RelayNotification::Shutdown => bail!("relay shut down during query"),
            _ => Ok(None),
        }
    }

    async fn handle_before(
        &mut self,
        deadline: Instant,
        notification: RelayNotification,
    ) -> anyhow::Result<Option<QueryCompletion>> {
        match timeout_at(deadline, self.handle(notification)).await {
            Ok(result) => result,
            Err(_) if self.delivering_progress => Err(progress::LocalBackpressure.into()),
            Err(_) => bail!("query timed out"),
        }
    }

    async fn handle_message(
        &mut self,
        message: RelayMessage,
    ) -> anyhow::Result<Option<QueryCompletion>> {
        match message {
            RelayMessage::Event {
                subscription_id,
                event,
            } if subscription_id == self.id => self.handle_event(*event).await,
            RelayMessage::EndOfStoredEvents(id) if id == self.id && !self.awaiting_auth => {
                Ok(Some(QueryCompletion {
                    authoritative: !self.lagged && !self.validation.overflowed(),
                }))
            }
            RelayMessage::Closed {
                subscription_id,
                message,
            } if subscription_id == self.id => self.handle_closed(&message),
            _ => Ok(None),
        }
    }

    async fn handle_event(&mut self, event: Event) -> anyhow::Result<Option<QueryCompletion>> {
        if self.validation.accept(&event)? {
            self.delivering_progress = true;
            self.sink.record(event).await;
            self.delivering_progress = false;
        }
        Ok(None)
    }
}
