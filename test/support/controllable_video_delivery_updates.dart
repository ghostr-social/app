import 'dart:async';

import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

final class ControllableVideoDeliveryUpdates implements VideoDeliveryUpdates {
  final _events = StreamController<VideoDeliverySnapshot>.broadcast(sync: true);

  @override
  Stream<VideoDeliverySnapshot> watchDelivery() => _events.stream;

  void publish(
    VideoPost post, {
    required bool startable,
    int? etaMilliseconds,
  }) {
    _events.add(
      VideoDeliverySnapshot(
        deliveryId: post.media.playbackDeliveryId!,
        phase: startable
            ? VideoDeliveryPhase.startable
            : VideoDeliveryPhase.preparing,
        bytesPresent: BigInt.zero,
        eta: etaMilliseconds == null
            ? null
            : Duration(milliseconds: etaMilliseconds),
      ),
    );
  }

  Future<void> close() => _events.close();
}
