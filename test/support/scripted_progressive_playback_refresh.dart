import 'dart:async';

import 'package:ghostr/core/media/progressive_playback_refresh_port.dart';
import 'package:ghostr/core/media/video_media_source.dart';

final class ScriptedProgressivePlaybackRefresh
    implements ProgressivePlaybackRefreshPort {
  final _pending = <Completer<ProxiedProgressiveVideoMediaSource>>[];

  int get requestCount => _pending.length;

  @override
  Future<ProxiedProgressiveVideoMediaSource> refresh() {
    final request = Completer<ProxiedProgressiveVideoMediaSource>();
    _pending.add(request);
    return request.future;
  }

  void completeNext(String playbackUrl) {
    _next.complete(ProxiedProgressiveVideoMediaSource(playbackUrl));
  }

  void completeAt(int index, String playbackUrl) {
    _pending[index].complete(ProxiedProgressiveVideoMediaSource(playbackUrl));
  }

  void failNext() {
    _next.completeError(StateError('Capability refresh failed'));
  }

  Completer<ProxiedProgressiveVideoMediaSource> get _next {
    return _pending.firstWhere((request) => !request.isCompleted);
  }
}
