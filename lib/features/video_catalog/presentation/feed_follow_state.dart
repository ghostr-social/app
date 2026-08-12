import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

final class FeedFollowState {
  factory FeedFollowState.unavailable({ProfileId? viewerId}) {
    return FeedFollowState._(viewerId, null, const <ProfileId>{});
  }

  factory FeedFollowState.ready({
    required ProfileId? viewerId,
    required Set<ProfileId> followed,
  }) {
    return FeedFollowState._(
      viewerId,
      Set<ProfileId>.unmodifiable(followed),
      const <ProfileId>{},
    );
  }

  FeedFollowState._(this.viewerId, this._followed, Set<ProfileId> pending)
    : _pending = Set<ProfileId>.unmodifiable(pending);

  final ProfileId? viewerId;
  final Set<ProfileId>? _followed;
  final Set<ProfileId> _pending;

  bool canFollow(ProfileId creatorId) {
    final followed = _followed;
    return followed != null &&
        viewerId != null &&
        creatorId != viewerId &&
        !followed.contains(creatorId) &&
        !_pending.contains(creatorId);
  }

  FeedFollowState starting(ProfileId creatorId) {
    return _copy(pending: {..._pending, creatorId});
  }

  FeedFollowState accepted(ProfileId creatorId) {
    final followed = _followed;
    if (followed == null) return this;
    return _copy(
      followed: {...followed, creatorId},
      pending: _without(creatorId),
    );
  }

  FeedFollowState rejected(ProfileId creatorId) {
    return _copy(pending: _without(creatorId));
  }

  FeedFollowState refreshed(Set<ProfileId> followed) {
    return _copy(followed: Set<ProfileId>.unmodifiable(followed));
  }

  FeedFollowState _copy({Set<ProfileId>? followed, Set<ProfileId>? pending}) {
    return FeedFollowState._(
      viewerId,
      followed ?? _followed,
      pending ?? _pending,
    );
  }

  Set<ProfileId> _without(ProfileId creatorId) {
    return {..._pending}..remove(creatorId);
  }
}
