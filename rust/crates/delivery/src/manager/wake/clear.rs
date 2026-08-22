use tokio::sync::oneshot;

pub(super) type ClearCompletion = (oneshot::Sender<anyhow::Result<()>>, anyhow::Result<()>);

pub(super) fn complete(clear: Option<ClearCompletion>) {
    if let Some((reply, result)) = clear {
        let _ = reply.send(result);
    }
}
