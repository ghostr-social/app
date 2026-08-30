import 'dart:async';

import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

final class ControlledVideoDeliveryUpdates implements VideoDeliveryUpdates {
  final _events = StreamController<VideoDeliverySnapshot>.broadcast(sync: true);

  @override
  Stream<VideoDeliverySnapshot> watchDelivery() => _events.stream;

  void publish(
    VideoPost post, {
    required VideoDeliveryPhase phase,
    Duration? eta,
    HlsPlaybackAuthority? hlsAuthority,
  }) {
    _events.add(
      VideoDeliverySnapshot(
        deliveryId: post.media.playbackDeliveryId!,
        phase: phase,
        bytesPresent: BigInt.zero,
        eta: eta,
        hlsAuthority: hlsAuthority,
      ),
    );
  }

  Future<void> close() => _events.close();
}
