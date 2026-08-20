import 'dart:async';

import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/progressive_playback_gateway_port.dart';

const fakeProgressivePlaybackUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';

class FakeProgressivePlaybackGateway implements ProgressivePlaybackGatewayPort {
  FakeProgressivePlaybackGateway({this.immediatePlaybackUrl});

  String? immediatePlaybackUrl;
  final requests = <VideoMediaSource>[];
  final _pending = <Completer<ProxiedProgressiveVideoMediaSource>>[];

  @override
  Future<ProxiedProgressiveVideoMediaSource> resolve(VideoMediaSource media) {
    requests.add(media);
    final immediate = immediatePlaybackUrl;
    if (immediate != null) {
      return Future.value(ProxiedProgressiveVideoMediaSource(immediate));
    }
    final pending = Completer<ProxiedProgressiveVideoMediaSource>();
    _pending.add(pending);
    return pending.future;
  }

  void completeNext({String playbackUrl = fakeProgressivePlaybackUrl}) {
    _nextPending.complete(ProxiedProgressiveVideoMediaSource(playbackUrl));
  }

  void resolveImmediatelyWith(String playbackUrl) {
    immediatePlaybackUrl = playbackUrl;
  }

  void failNext() {
    _nextPending.completeError(StateError('Gateway unavailable'));
  }

  Completer<ProxiedProgressiveVideoMediaSource> get _nextPending {
    return _pending.firstWhere((pending) => !pending.isCompleted);
  }
}
