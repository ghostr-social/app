import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';

import 'device_playback_probe.dart';
import 'live_video_log.dart';
import 'warp_feed_focus_probe.dart';

final class LiveFocusProbe implements FeedFocusSink {
  LiveFocusProbe(DevicePlaybackProbe playback, this.log) {
    final native = FfiFeedFocusPort();
    probe = WarpFeedFocusProbe(
      native,
      playback,
      () => native.lastScheduledGeneration,
    );
  }

  final LiveVideoLog log;
  late final WarpFeedFocusProbe probe;
  final posts = <String, VideoPost>{};
  FeedFocus? current;

  @override
  void clearFocus() => probe.clearFocus();

  @override
  void focusChanged(FeedFocus focus) {
    current = focus;
    for (final post in focus.window) {
      if (posts.containsKey(post.id.value)) continue;
      posts[post.id.value] = post;
      log.add('post', {
        'eventId': post.id.value,
        'kind': post.nostrReference?.kind.value,
        'url': post.media.remoteUrl,
        'fallbacks': post.media.fallbackUrls,
        'delivery': post.media.remoteDelivery?.name,
      });
    }
    probe.focusChanged(focus);
    final occurrence = probe.occurrences.last;
    log.add('focus', {
      'eventId': focus.current.id.value,
      'cause': focus.cause.name,
      'sequence': occurrence.sequence,
      'generation': probe.generationFor(occurrence)?.toString(),
      'deliveryId': probe.deliveryForEvent(focus.current.id.value)?.value,
      'rescue': focus.rescue?.reason.name,
    });
  }
}
