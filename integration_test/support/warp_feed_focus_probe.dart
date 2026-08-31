import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

import 'device_playback_probe.dart';

typedef WarpFeedFocusGeneration = BigInt? Function();

final class WarpFeedFocusProbe implements FeedFocusPort {
  WarpFeedFocusProbe(this._delegate, this._playback, [this._generation]);

  final FeedFocusPort _delegate;
  final DevicePlaybackProbe _playback;
  final WarpFeedFocusGeneration? _generation;
  final _published = <String, PlaybackFocus>{};
  final _occurrences = <PlaybackFocus>[];
  final _generations = <int, BigInt>{};
  final _deliveries = <String, PlaybackDeliveryId>{};

  List<PlaybackFocus> get occurrences => List.unmodifiable(_occurrences);

  bool get hadTransportRescue => _occurrences.any(
    (focus) => focus.cause == FeedFocusCause.transportRescue,
  );

  @override
  void focusChanged(FeedFocus focus) {
    _rememberDeliveries(focus);
    final id = PlaybackVideoId.parse(focus.current.id);
    final occurrence = _playback.markFocus(
      id,
      cause: focus.cause,
      rescue: focus.rescue,
    );
    _published[id.value] = occurrence;
    _occurrences.add(occurrence);
    _delegate.focusChanged(focus);
    final generation = _generation?.call();
    if (generation != null) _generations[occurrence.sequence] = generation;
  }

  PlaybackFocus? publishedFor(String eventId) => _published[eventId];

  PlaybackDeliveryId? deliveryForEvent(String eventId) => _deliveries[eventId];

  BigInt? generationFor(PlaybackFocus occurrence) {
    return _generations[occurrence.sequence];
  }

  List<PlaybackFocus> occurrencesFor(String eventId, {FeedFocusCause? cause}) {
    return List.unmodifiable(
      _occurrences.where(
        (item) =>
            item.videoId.value == eventId &&
            (cause == null || item.cause == cause),
      ),
    );
  }

  PlaybackFocus? occurrenceAfter(
    String eventId,
    int sequence, {
    FeedFocusCause? cause,
  }) {
    for (final item in _occurrences) {
      if (item.sequence <= sequence || item.videoId.value != eventId) continue;
      if (cause == null || item.cause == cause) return item;
    }
    return null;
  }

  void _rememberDeliveries(FeedFocus focus) {
    for (final post in focus.window) {
      final deliveryId = post.media.playbackDeliveryId;
      if (deliveryId != null) _deliveries[post.id.value] = deliveryId;
    }
  }
}
