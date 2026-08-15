import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

/// Selects only inside the paper's small semantic display window.
final class FeedReadySelector {
  const FeedReadySelector({this.maxCandidates = 3}) : assert(maxCandidates > 0);

  final int maxCandidates;

  int select(
    List<VideoPost> posts, {
    required int fromIndex,
    required int intendedIndex,
    required Map<PlaybackDeliveryId, VideoDeliverySnapshot> delivery,
  }) {
    final intended = _snapshot(posts[intendedIndex], delivery);
    if (intended == null || intended.phase == VideoDeliveryPhase.startable) {
      return intendedIndex;
    }
    final direction = intendedIndex.compareTo(fromIndex);
    if (direction == 0) return intendedIndex;
    return _firstReady(posts, intendedIndex, direction, delivery);
  }

  int _firstReady(
    List<VideoPost> posts,
    int intended,
    int direction,
    Map<PlaybackDeliveryId, VideoDeliverySnapshot> delivery,
  ) {
    for (var distance = 1; distance < maxCandidates; distance += 1) {
      final index = intended + (distance * direction);
      if (index < 0 || index >= posts.length) break;
      final candidate = _snapshot(posts[index], delivery);
      if (candidate?.phase == VideoDeliveryPhase.startable) return index;
    }
    return intended;
  }
}

VideoDeliverySnapshot? _snapshot(
  VideoPost post,
  Map<PlaybackDeliveryId, VideoDeliverySnapshot> delivery,
) {
  final id = post.media.playbackDeliveryId;
  return id == null ? null : delivery[id];
}
