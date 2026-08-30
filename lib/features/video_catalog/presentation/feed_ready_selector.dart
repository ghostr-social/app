import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';

enum FeedReadyAction { intended, wait, rescue }

enum FeedReadyReason {
  intendedReady,
  historyTraversal,
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

final class FeedReadinessEvidence {
  const FeedReadinessEvidence({
    required this.posts,
    required this.delivery,
    this.preparation,
    this.verifiedHlsAuthorities = const {},
  });

  final List<VideoPost> posts;
  final Map<PlaybackDeliveryId, VideoDeliverySnapshot> delivery;
  final FeedPlaybackPreparation? preparation;
  final Set<HlsPlaybackAuthority> verifiedHlsAuthorities;

  VideoDeliverySnapshot? snapshotAt(int index) {
    final deliveryId = posts[index].media.playbackDeliveryId;
    return deliveryId == null ? null : delivery[deliveryId];
  }

  bool isStructurallyStartableAt(int index) {
    final snapshot = snapshotAt(index);
    if (snapshot?.phase == VideoDeliveryPhase.failed &&
        _failureApplies(index, snapshot!)) {
      return false;
    }
    if (snapshot?.phase == VideoDeliveryPhase.startable) return true;
    return preparation?.isStructurallyStartable(posts[index].media) == true;
  }

  bool isPlayerVerifiedAt(int index) {
    final snapshot = snapshotAt(index);
    if (snapshot?.phase == VideoDeliveryPhase.failed) return false;
    final hlsAuthority = snapshot?.hlsAuthority;
    if (hlsAuthority != null) {
      return verifiedHlsAuthorities.contains(hlsAuthority) &&
          hlsAuthority.deliveryId == posts[index].media.playbackDeliveryId &&
          hlsAuthority.representationId ==
              VideoRepresentationId.forMedia(posts[index].media);
    }
    final prepared = preparation?.forMedia(posts[index].media);
    if (prepared?.readiness.isPlayerVerified != true) return false;
    final deliveryAuthority = snapshot?.authority;
    return deliveryAuthority == null ||
        deliveryAuthority == prepared!.authority;
  }

  bool _failureApplies(int index, VideoDeliverySnapshot snapshot) {
    final authority = snapshot.authority;
    if (authority == null) return true;
    final prepared = preparation?.forMedia(posts[index].media);
    return prepared == null || prepared.authority == authority;
  }
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
    FeedReadinessEvidence evidence, {
    required int fromIndex,
    required int intendedIndex,
    bool graceExpired = false,
  }) {
    final intended = evidence.snapshotAt(intendedIndex);
    if (evidence.isPlayerVerifiedAt(intendedIndex)) {
      return _stay(intendedIndex, FeedReadyReason.intendedReady);
    }
    if (intendedIndex < fromIndex) {
      return _stay(intendedIndex, FeedReadyReason.historyTraversal);
    }
    if (intended == null) {
      return _stay(intendedIndex, FeedReadyReason.unknownDelivery);
    }
    final ready = _firstReady(evidence, fromIndex, intendedIndex);
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

  int _firstReady(FeedReadinessEvidence evidence, int from, int intended) {
    final direction = intended.compareTo(from);
    if (direction == 0) return intended;
    for (var distance = 1; distance < maxCandidates; distance += 1) {
      final index = intended + (distance * direction);
      if (index < 0 || index >= evidence.posts.length) break;
      if (evidence.isPlayerVerifiedAt(index)) return index;
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
