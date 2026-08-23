import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

import 'device_playback_probe.dart';

final class WarpFeedFocusProbe implements FeedFocusPort {
  WarpFeedFocusProbe(this._delegate, this._playback);

  final FeedFocusPort _delegate;
  final DevicePlaybackProbe _playback;
  final _published = <String, PlaybackFocus>{};

  @override
  void focusChanged(FeedFocus focus) {
    final id = PlaybackVideoId.parse(focus.current.id);
    _published[id.value] = _playback.markFocus(id);
    _delegate.focusChanged(focus);
  }

  PlaybackFocus? publishedFor(String eventId) => _published[eventId];
}
