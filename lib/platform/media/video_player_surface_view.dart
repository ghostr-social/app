import 'package:flutter/material.dart';
import 'package:ghostr/core/media/inline_blurhash.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/video_loading_surface.dart';
import 'package:video_player/video_player.dart';

class VideoPlayerSurfaceView extends StatelessWidget {
  const VideoPlayerSurfaceView({
    required this.controller,
    required this.hasError,
    required this.onRetry,
    this.preview,
    super.key,
  });

  final VideoPlayerController? controller;
  final bool hasError;
  final VoidCallback onRetry;
  final InlineBlurHash? preview;

  @override
  Widget build(BuildContext context) {
    if (hasError) return _error();
    final activeController = controller;
    if (activeController == null || !activeController.value.isInitialized) {
      return VideoLoadingSurface(label: 'Loading video', preview: preview);
    }
    return _ready(activeController);
  }

  Widget _ready(VideoPlayerController activeController) {
    return ColoredBox(
      color: AppPalette.videoBackground,
      child: FittedBox(
        fit: BoxFit.cover,
        child: SizedBox(
          width: activeController.value.size.width,
          height: activeController.value.size.height,
          child: VideoPlayer(activeController),
        ),
      ),
    );
  }

  Widget _error() {
    return AsyncStatePanel(
      icon: Icons.play_disabled_outlined,
      title: 'Video unavailable',
      message: 'Ghostr could not start this video.',
      actionLabel: 'Retry',
      onAction: onRetry,
    );
  }
}
