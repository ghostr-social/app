import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';

final class HlsPlaybackLease {
  HlsPlaybackLease({
    required this.deliveryId,
    required this.media,
    required void Function() onReleased,
  }) : _onReleased = onReleased;

  final PlaybackDeliveryId deliveryId;
  final ProxiedHlsVideoMediaSource media;
  final void Function() _onReleased;
  bool _released = false;

  void release() {
    if (_released) return;
    _released = true;
    _onReleased();
  }
}
