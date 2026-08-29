import 'package:flutter/widgets.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

final class InactivePreparedPlaybackGate implements VideoPlaybackPort {
  InactivePreparedPlaybackGate(this._delegate, this._blockedVideoId);

  final VideoPlaybackPort _delegate;
  final PlaybackVideoId _blockedVideoId;
  var blockedBuilds = 0;

  @override
  Widget buildSurface(VideoPlaybackSurfaceRequest request) {
    if (!_blocks(request)) return _delegate.buildSurface(request);
    blockedBuilds += 1;
    return const SizedBox.expand();
  }

  bool _blocks(VideoPlaybackSurfaceRequest request) {
    return !request.isActive &&
        request.reservesPreparedDecoder &&
        request.videoId == _blockedVideoId;
  }
}
