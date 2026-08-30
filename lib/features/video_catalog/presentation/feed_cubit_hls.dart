part of 'feed_cubit.dart';

extension FeedCubitHls on FeedCubit {
  void hlsFirstFrameRendered(HlsPlaybackAuthority authority) {
    _hlsPlayerVerificationChanged(authority, true);
  }

  void hlsPlaybackReleased(HlsPlaybackAuthority authority) {
    _hlsPlayerVerificationChanged(authority, false);
  }

  FeedLoaded _projectHlsDelivery(FeedLoaded feed) {
    var projected = feed;
    for (final post in feed.posts) {
      final deliveryId = post.media.playbackDeliveryId;
      if (deliveryId == null) continue;
      final snapshot = _delivery[deliveryId];
      if (snapshot == null) continue;
      projected = projected.withHlsAuthority(deliveryId, snapshot.hlsAuthority);
    }
    return projected;
  }

  FeedLoaded _realignHls(FeedLoaded previous, FeedLoaded moved) {
    var projected = _projectHlsDelivery(moved);
    for (final authority in previous.verifiedHlsAuthorities) {
      projected = projected.withVerifiedHls(authority, true);
    }
    return projected;
  }

  void _hlsPlayerVerificationChanged(
    HlsPlaybackAuthority authority,
    bool verified,
  ) {
    final current = state;
    if (current is! FeedLoaded) return;
    final updated = current.withVerifiedHls(authority, verified);
    if (identical(updated, current)) return;
    emit(updated);
    if (verified) _rescueAfterDeliveryUpdate();
  }
}
