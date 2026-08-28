import 'dart:async';

import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_lease.dart';
import 'package:ghostr/core/media/video_media_source.dart';

class FakeHlsPlaybackGateway implements HlsPlaybackGatewayPort {
  final requests = <HlsPlaybackRequest>[];
  final _pending = <Completer<HlsPlaybackLease>>[];
  int activeLeaseCount = 0;

  @override
  Future<HlsPlaybackLease> acquire(HlsPlaybackRequest request) {
    requests.add(request);
    final pending = Completer<HlsPlaybackLease>();
    _pending.add(pending);
    return pending.future;
  }

  void completeNext({
    String proxyUrl = _proxyUrl,
    PlaybackDeliveryId? deliveryId,
  }) {
    completeAt(
      _pending.indexWhere((pending) => !pending.isCompleted),
      proxyUrl: proxyUrl,
      deliveryId: deliveryId,
    );
  }

  void completeAt(
    int index, {
    String proxyUrl = _proxyUrl,
    PlaybackDeliveryId? deliveryId,
  }) {
    activeLeaseCount += 1;
    _pending[index].complete(
      HlsPlaybackLease(
        deliveryId: deliveryId ?? requests[index].deliveryId,
        media: ProxiedHlsVideoMediaSource(proxyUrl),
        onReleased: () => activeLeaseCount -= 1,
      ),
    );
  }

  void failNext() => _nextPending.completeError(StateError('HLS unavailable'));

  Completer<HlsPlaybackLease> get _nextPending {
    return _pending.firstWhere((pending) => !pending.isCompleted);
  }
}

const _proxyUrl =
    'http://127.0.0.1:3210/hls/'
    '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef/'
    'index.m3u8';
