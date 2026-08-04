import 'package:ghostr/features/video_catalog/data/shadow_compare_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';

/// Builds the Rust discovery pipeline on demand. It stays a thunk so
/// the default build never touches the engine's feed FFI.
typedef RustFeedSourceBuilder = RemoteVideoSource Function();

/// Which discovery pipeline serves the app's feeds (plan §5 step 6).
enum FeedPipelineMode {
  /// The shipping ndk relay path.
  ndk,

  /// Rust discovery alone.
  rust,

  /// ndk serves the viewer; Rust runs alongside and parity is logged.
  shadow,
}

/// The migration switch between the two discovery pipelines.
///
/// ndk is the default and the only mode that ships today: choosing it
/// must never build the Rust source, so a half-wired engine cannot
/// affect the shipping path.
final class FeedPipelineFlag {
  const FeedPipelineFlag([this.mode = FeedPipelineMode.ndk]);

  final FeedPipelineMode mode;

  RemoteVideoSource select({
    required RemoteVideoSource ndk,
    required RustFeedSourceBuilder rust,
  }) {
    return switch (mode) {
      FeedPipelineMode.ndk => ndk,
      FeedPipelineMode.rust => rust(),
      FeedPipelineMode.shadow => ShadowCompareRemoteVideoSource(
          primary: ndk,
          shadow: rust(),
        ),
    };
  }
}
