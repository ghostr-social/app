import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_media_identity.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

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
  FeedRoster resynced(List<VideoPost> refreshed, {List<VideoPost>? eligible}) {
    final byTarget = <VideoInteractionTarget, VideoPost>{
      for (final post in refreshed) _targetOf(post): post,
    };
    final kept = <VideoPost>[
      for (final post in posts)
        if (byTarget[_targetOf(post)] case final revision?) revision,
    ];
    final seen = SeenVideoIdentities(kept);
    final merged = <VideoPost>[
      ...kept,
      for (final post in eligible ?? refreshed)
        if (seen.add(post)) post,
    ];
    return FeedRoster(merged, activeIndex: _preserved(merged));
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
