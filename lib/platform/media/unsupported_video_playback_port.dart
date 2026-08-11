import 'package:flutter/material.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';

final class UnsupportedVideoPlaybackPort implements VideoPlaybackPort {
  const UnsupportedVideoPlaybackPort();

  @override
  Widget buildSurface({
    required VideoMediaSource media,
    PlaybackVideoId? videoId,
    required bool isActive,
    void Function()? onPlaybackMediaReleased,
  }) {
    return _UnsupportedVideoPlaybackSurface(
      key: ValueKey(media.inventoryPlaybackIdentity),
      onPlaybackMediaReleased: onPlaybackMediaReleased,
    );
  }
}

final class _UnsupportedVideoPlaybackSurface extends StatefulWidget {
  const _UnsupportedVideoPlaybackSurface({
    required this.onPlaybackMediaReleased,
    super.key,
  });

  final VoidCallback? onPlaybackMediaReleased;

  @override
  State<_UnsupportedVideoPlaybackSurface> createState() =>
      _UnsupportedVideoPlaybackSurfaceState();
}

final class _UnsupportedVideoPlaybackSurfaceState
    extends State<_UnsupportedVideoPlaybackSurface> {
  @override
  void dispose() {
    widget.onPlaybackMediaReleased?.call();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return const AsyncStatePanel(
      icon: Icons.play_disabled_outlined,
      title: 'Video playback unavailable',
      message: 'This platform has no compatible video player.',
    );
  }
}
