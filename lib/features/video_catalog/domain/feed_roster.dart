import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_navigation_history.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_media_identity.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

export 'package:ghostr/features/video_catalog/domain/feed_navigation_history.dart';

/// The posts a viewer is scrolling through and where they are standing in
/// them.
///
/// Content shifts under a live feed: a refresh drops posts the relays no
/// longer return, a block removes a creator mid-scroll. The roster decides
/// what survives and which video keeps playing, so the viewer is never
/// teleported to a different post.
final class FeedRoster {
  const FeedRoster(this.posts, {this.activeIndex = 0});

  final List<VideoPost> posts;
  final int activeIndex;

  bool get isEmpty => posts.isEmpty;

  VideoPost get active => posts[activeIndex];

  /// How many videos are still queued after the one on screen.
  int get ahead => posts.length - activeIndex - 1;

  /// The viewer enters the feed standing on [postId] — the video they chose
  /// on a profile grid. Opens at the top when the post is absent.
  FeedRoster openedAt(VideoPostId? postId) {
    if (postId == null) return this;
    final index = posts.indexWhere((post) => post.id == postId);
    return index < 0 ? this : FeedRoster(posts, activeIndex: index);
  }

  /// Replaces held revisions in place, drops vanished posts, and appends
  /// newly eligible rows after the held tail.
  FeedRoster resynced(
    List<VideoPost> refreshed, {
    List<VideoPost>? eligible,
    bool retainWatched = true,
    Set<VideoInteractionTarget> retainedHeldTargets = const {},
  }) {
    final eligibility = eligible ?? refreshed;
    final retained = <VideoInteractionTarget>{
      ...retainedHeldTargets,
      if (!isEmpty) _targetOf(active),
    };
    final admission = _RefreshAdmission(
      refreshed,
      eligibility,
      retainWatched,
      retained,
    );
    final kept = _keptPosts(admission);
    final seen = SeenVideoIdentities(kept);
    final merged = <VideoPost>[
      ...kept,
      for (final post in eligibility)
        if (seen.add(post)) post,
    ];
    return FeedRoster(merged, activeIndex: _preserved(merged));
  }

  List<VideoPost> _keptPosts(_RefreshAdmission admission) {
    return <VideoPost>[
      for (final post in posts)
        if (admission.revisionFor(post) case final revision?) revision,
    ];
  }

  FeedRoster movedTo(int index, {required FeedNavigationHistory history}) {
    RangeError.checkValidIndex(index, posts, 'index');
    final first = history.firstRetained(index);
    return FeedRoster(
      List<VideoPost>.unmodifiable(posts.skip(first)),
      activeIndex: index - first,
    );
  }

  /// Drops every post published by [creator].
  FeedRoster withoutCreator(ProfileId creator) {
    final remaining = <VideoPost>[
      for (final post in posts)
        if (post.creator.id != creator) post,
    ];
    return FeedRoster(remaining, activeIndex: _surviving(remaining));
  }

  /// The same video keeps playing; once it is gone the viewer holds their
  /// position and clamps to the end of what is left.
  int _preserved(List<VideoPost> kept) {
    if (isEmpty || kept.isEmpty) return 0;
    final index = _indexOf(kept, _targetOf(active));
    if (index >= 0) return index;
    final lastIndex = kept.length - 1;
    return activeIndex > lastIndex ? lastIndex : activeIndex;
  }

  /// The first survivor at or below the viewer's position keeps playing.
  int _surviving(List<VideoPost> remaining) {
    if (remaining.isEmpty) return 0;
    for (var index = activeIndex; index < posts.length; index += 1) {
      final found = _indexOf(remaining, _targetOf(posts[index]));
      if (found >= 0) return found;
    }
    return remaining.length - 1;
  }

  static int _indexOf(List<VideoPost> posts, VideoInteractionTarget target) {
    return posts.indexWhere((post) => _targetOf(post) == target);
  }

  static VideoInteractionTarget _targetOf(VideoPost post) {
    return VideoInteractionTarget.fromPost(post);
  }
}

typedef _RefreshRevisions = ({
  Map<VideoInteractionTarget, VideoPost> raw,
  Map<VideoInteractionTarget, VideoPost> eligible,
});

final class _RefreshAdmission {
  factory _RefreshAdmission(
    List<VideoPost> raw,
    List<VideoPost> eligible,
    bool retainWatched,
    Set<VideoInteractionTarget> retainedTargets,
  ) {
    return _RefreshAdmission._(
      (raw: _index(raw), eligible: _index(eligible)),
      retainWatched,
      retainedTargets,
    );
  }

  const _RefreshAdmission._(
    this.revisions,
    this.retainWatched,
    this.retainedTargets,
  );

  final _RefreshRevisions revisions;
  final bool retainWatched;
  final Set<VideoInteractionTarget> retainedTargets;

  VideoPost? revisionFor(VideoPost held) {
    final target = VideoInteractionTarget.fromPost(held);
    final admitted = retainWatched ? revisions.raw : revisions.eligible;
    if (admitted[target] case final revision?) return revision;
    if (!retainedTargets.contains(target)) return null;
    return revisions.raw.containsKey(target) ? held : null;
  }

  static Map<VideoInteractionTarget, VideoPost> _index(List<VideoPost> posts) {
    return <VideoInteractionTarget, VideoPost>{
      for (final post in posts) VideoInteractionTarget.fromPost(post): post,
    };
  }
}
