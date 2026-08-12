import 'package:flutter/widgets.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

class RecordingVideoPlaybackPort implements VideoPlaybackPort {
  final Map<String, List<bool>> activity = {};

  @override
  Widget buildSurface(VideoPlaybackSurfaceRequest request) {
    (activity[request.media.debugLabel] ??= []).add(request.isActive);
    return _ReleaseOnDispose(
      onReleased: request.onPlaybackMediaReleased,
      child: const SizedBox.expand(),
    );
  }
}

class _ReleaseOnDispose extends StatefulWidget {
  const _ReleaseOnDispose({required this.onReleased, required this.child});

  final void Function()? onReleased;
  final Widget child;

  @override
  State<_ReleaseOnDispose> createState() => _ReleaseOnDisposeState();
}

class _ReleaseOnDisposeState extends State<_ReleaseOnDispose> {
  @override
  Widget build(BuildContext context) => widget.child;

  @override
  void dispose() {
    widget.onReleased?.call();
    super.dispose();
  }
}
