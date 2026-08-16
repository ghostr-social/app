import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_media_identity.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_policy.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

/// The posts a viewer is scrolling through and where they are standing in
/// them.
///
/// Passive snapshots can be partial, so admitted posts stay pinned until an
/// explicit action removes them. The roster also preserves which video keeps
/// playing, so the viewer is never teleported to a different post.
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

  /// Replaces safe held revisions in place and appends newly eligible rows
  /// after the pinned tail.
  FeedRoster resynced(List<VideoPost> refreshed, {List<VideoPost>? eligible}) {
    final eligibility = eligible ?? refreshed;
    final admission = _RefreshAdmission(refreshed, eligibility);
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
    return <VideoPost>[for (final post in posts) admission.revisionFor(post)];
  }

  FeedRoster movedTo(int index) {
    RangeError.checkValidIndex(index, posts, 'index');
    return FeedRoster(posts, activeIndex: index);
  }

  /// Drops every post published or reposted by [creator].
  FeedRoster withoutCreator(ProfileId creator) {
    return withoutBlocked({creator});
  }

  /// Drops posts whose creator or reposter is explicitly blocked.
  FeedRoster withoutBlocked(Set<ProfileId> blocked) {
    final remaining = <VideoPost>[
      for (final post in posts)
        if (!isBlockedVideoPost(post, blocked)) post,
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

final class _RefreshAdmission {
  factory _RefreshAdmission(List<VideoPost> raw, List<VideoPost> eligible) {
    final rawByTarget = _index(raw);
    final eligibleByTarget = _index(eligible);
    return _RefreshAdmission._(rawByTarget, eligibleByTarget);
  }

  const _RefreshAdmission._(this.rawByTarget, this.eligibleByTarget);

  final Map<VideoInteractionTarget, VideoPost> rawByTarget;
  final Map<VideoInteractionTarget, VideoPost> eligibleByTarget;

  VideoPost revisionFor(VideoPost held) {
    final target = VideoInteractionTarget.fromPost(held);
    if (eligibleByTarget[target] case final revision?
        when sharesVideoMedia(held, revision)) {
      return revision;
    }
    final raw = rawByTarget[target];
    return raw != null && sharesVideoMedia(held, raw) ? raw : held;
  }

  static Map<VideoInteractionTarget, VideoPost> _index(List<VideoPost> posts) {
    return <VideoInteractionTarget, VideoPost>{
      for (final post in posts) VideoInteractionTarget.fromPost(post): post,
    };
  }
}
