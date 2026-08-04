import 'package:ghostr/app/feed_pipeline_flag.dart';
import 'package:ghostr/features/social/domain/signed_event_broadcast_port.dart';

/// Builds the Rust transport on demand, so the shipping path never
/// touches the engine's broadcast FFI.
typedef RustBroadcastPortBuilder = SignedEventBroadcastPort Function();

/// Picks the transport for social writes from the discovery pipeline in
/// use (plan §5 steps 5 and 6).
///
/// Shadow mode reads from both pipelines but writes stay on ndk: a
/// parity run must never publish an event twice.
SignedEventBroadcastPort selectBroadcastTransport({
  required FeedPipelineMode mode,
  required SignedEventBroadcastPort ndk,
  required RustBroadcastPortBuilder rust,
}) {
  return switch (mode) {
    FeedPipelineMode.ndk || FeedPipelineMode.shadow => ndk,
    FeedPipelineMode.rust => rust(),
  };
}
