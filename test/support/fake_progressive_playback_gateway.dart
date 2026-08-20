import 'dart:async';

import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/progressive_playback_gateway_port.dart';

const fakeProgressivePlaybackUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';

class FakeProgressivePlaybackGateway implements ProgressivePlaybackGatewayPort {
  FakeProgressivePlaybackGateway({this.immediatePlaybackUrl});

  String? immediatePlaybackUrl;
  final requests = <VideoMediaSource>[];
  final _pending = <_PendingResolution>[];

  @override
  Future<ProxiedProgressiveVideoMediaSource> resolve(VideoMediaSource media) {
    requests.add(media);
    final immediate = immediatePlaybackUrl;
    if (immediate != null) {
      return Future.value(ProxiedProgressiveVideoMediaSource(immediate));
    }
    final pending = Completer<ProxiedProgressiveVideoMediaSource>();
    _pending.add((media: media, result: pending));
    return pending.future;
  }

  void completeNext({String? playbackUrl}) {
    final pending = _nextPending;
    pending.result.complete(
      ProxiedProgressiveVideoMediaSource(
        playbackUrl ?? _playbackUrl(pending.media),
      ),
    );
  }

  void resolveImmediatelyWith(String playbackUrl) {
    immediatePlaybackUrl = playbackUrl;
  }

  void failNext() {
    _nextPending.result.completeError(StateError('Gateway unavailable'));
  }

  _PendingResolution get _nextPending {
    return _pending.firstWhere((pending) => !pending.result.isCompleted);
  }
}

typedef _PendingResolution = ({
  VideoMediaSource media,
  Completer<ProxiedProgressiveVideoMediaSource> result,
});

String _playbackUrl(VideoMediaSource media) {
  final deliveryId = media.playbackDeliveryId;
  if (deliveryId == null) return fakeProgressivePlaybackUrl;
  final id = Uri.encodeQueryComponent(deliveryId.value);
  return 'http://127.0.0.1:3210/video.mp4?id=$id&cap='
      'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
}
