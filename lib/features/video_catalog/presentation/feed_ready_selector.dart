import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

enum FeedReadyAction { intended, wait, rescue }

enum FeedReadyReason {
  intendedReady,
  unknownDelivery,
  shortEta,
  etaTooLong,
  etaUnavailable,
  deliveryFailed,
  graceExpired,
  noReadyAlternative,
}

final class FeedReadyDecision {
  const FeedReadyDecision({
    required this.action,
    required this.reason,
    required this.intendedIndex,
    required this.selectedIndex,
  });

  final FeedReadyAction action;
  final FeedReadyReason reason;
  final int intendedIndex;
  final int selectedIndex;

  int get displacement => (selectedIndex - intendedIndex).abs();
}

/// Keeps feed semantics authoritative while avoiding a visibly stalled post.
final class FeedReadySelector {
  const FeedReadySelector({
    this.maxCandidates = 3,
    this.grace = const Duration(milliseconds: 250),
  }) : assert(maxCandidates > 0);

  final int maxCandidates;
  final Duration grace;

  FeedReadyDecision select(
    List<VideoPost> posts, {
    required int fromIndex,
    required int intendedIndex,
    required Map<PlaybackDeliveryId, VideoDeliverySnapshot> delivery,
    bool graceExpired = false,
  }) {
    final intended = _snapshot(posts[intendedIndex], delivery);
    if (intended == null) {
      return _stay(intendedIndex, FeedReadyReason.unknownDelivery);
    }
    if (intended.phase == VideoDeliveryPhase.startable) {
      return _stay(intendedIndex, FeedReadyReason.intendedReady);
    }
    final ready = _firstReady(posts, fromIndex, intendedIndex, delivery);
    final wait =
        intended.phase != VideoDeliveryPhase.failed &&
        intended.eta != null &&
        intended.eta! <= grace &&
        !graceExpired;
    if (wait) return _wait(intendedIndex);
    if (ready == intendedIndex) {
      return _stay(intendedIndex, FeedReadyReason.noReadyAlternative);
    }
    return _rescue(intended, intendedIndex, ready, graceExpired);
  }

  int _firstReady(
    List<VideoPost> posts,
    int from,
    int intended,
    Map<PlaybackDeliveryId, VideoDeliverySnapshot> delivery,
  ) {
    final direction = intended.compareTo(from);
    if (direction == 0) return intended;
    for (var distance = 1; distance < maxCandidates; distance += 1) {
      final index = intended + (distance * direction);
      if (index < 0 || index >= posts.length) break;
      if (_snapshot(posts[index], delivery)?.phase ==
          VideoDeliveryPhase.startable) {
        return index;
      }
    }
    return intended;
  }
}

FeedReadyDecision _stay(int index, FeedReadyReason reason) => FeedReadyDecision(
  action: FeedReadyAction.intended,
  reason: reason,
  intendedIndex: index,
  selectedIndex: index,
);

FeedReadyDecision _wait(int index) => FeedReadyDecision(
  action: FeedReadyAction.wait,
  reason: FeedReadyReason.shortEta,
  intendedIndex: index,
  selectedIndex: index,
);

FeedReadyDecision _rescue(
  VideoDeliverySnapshot intended,
  int intendedIndex,
  int selectedIndex,
  bool graceExpired,
) => FeedReadyDecision(
  action: FeedReadyAction.rescue,
  reason: graceExpired
      ? FeedReadyReason.graceExpired
      : intended.phase == VideoDeliveryPhase.failed
      ? FeedReadyReason.deliveryFailed
      : intended.eta == null
      ? FeedReadyReason.etaUnavailable
      : FeedReadyReason.etaTooLong,
  intendedIndex: intendedIndex,
  selectedIndex: selectedIndex,
);

VideoDeliverySnapshot? _snapshot(
  VideoPost post,
  Map<PlaybackDeliveryId, VideoDeliverySnapshot> delivery,
) {
  final id = post.media.playbackDeliveryId;
  return id == null ? null : delivery[id];
}
