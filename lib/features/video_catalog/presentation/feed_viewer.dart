import 'dart:async';

import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/domain/video_watch_fingerprints.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

/// Tells the rest of the app where the viewer is.
///
/// The delivery engine reprioritizes its downloads around the focus window,
/// and watch history remembers the video on screen so it is not served
/// again. Both are optional: a feed can run without either.
final class FeedViewer {
  FeedViewer({this.focus, this.watchTracker, this.onWatchFailure});

  final FeedFocusPort? focus;
  final WatchHistoryTracker? watchTracker;
  final void Function(Object error, StackTrace stackTrace)? onWatchFailure;
  final Set<Future<bool>> _pendingWrites = <Future<bool>>{};
  final Map<String, Future<bool>> _preparations = {};
  String? _visibleIdentity;
  String? _requestedIdentity;
  var _isDisposed = false;

  /// The viewer landed on `posts[index]` — a load, a swipe, a jump.
  void landedOn(List<VideoPost> posts, int index) {
    _watchedIfNew(posts[index]);
    _publish(posts, index, FeedFocusCause.userNavigation);
  }

  /// The visible roster changed while the viewer remained in place.
  void rosterChanged(List<VideoPost> posts, int index) {
    _watchedIfNew(posts[index]);
    _publish(posts, index, FeedFocusCause.rosterChange);
  }

  /// Delivery selected a playable neighbor; this is not a user preference.
  void rescuedTo(List<VideoPost> posts, int index, FeedTransportRescue rescue) {
    _watchedIfNew(posts[index]);
    _publish(posts, index, FeedFocusCause.transportRescue, rescue: rescue);
  }

  void visibilityChanged(bool isVisible) {
    final lease = focus;
    if (!isVisible) {
      _visibleIdentity = null;
      _requestedIdentity = null;
      if (lease is FeedFocusLease) lease.deactivate();
      return;
    }
    if (lease is FeedFocusLease) lease.activate();
  }

  /// Persists [post] before a feed is allowed to render it.
  FutureOr<bool> prepareToShow(VideoPost post) {
    if (_isDisposed) return false;
    final identity = _identityOf(post);
    if (_visibleIdentity == identity) return true;
    _requestedIdentity = identity;
    if (_preparations[identity] case final pending?) return pending;
    final tracker = watchTracker;
    if (tracker == null) {
      _visibleIdentity = identity;
      return true;
    }
    return _trackPreparation(tracker, post, identity);
  }

  Future<void> dispose() async {
    _isDisposed = true;
    final lease = focus;
    if (lease is FeedFocusLease) lease.release();
    await Future.wait(List<Future<bool>>.of(_pendingWrites));
  }

  void _publish(
    List<VideoPost> posts,
    int index,
    FeedFocusCause cause, {
    FeedTransportRescue? rescue,
  }) {
    // Reactivation carries no invented watch time.
    focus?.focusChanged(
      FeedFocus.around(
        posts: posts,
        activeIndex: index,
        cause: cause,
        rescue: rescue,
      ),
    );
  }

  void _watchedIfNew(VideoPost post) {
    final preparation = prepareToShow(post);
    if (preparation is bool) return;
    unawaited(preparation);
  }

  Future<bool> _trackPreparation(
    WatchHistoryTracker tracker,
    VideoPost post,
    String identity,
  ) {
    late final Future<bool> pending;
    pending = _record(tracker, post, identity).whenComplete(() {
      _preparations.remove(identity);
      _pendingWrites.remove(pending);
    });
    _preparations[identity] = pending;
    _pendingWrites.add(pending);
    return pending;
  }

  Future<bool> _record(
    WatchHistoryTracker tracker,
    VideoPost post,
    String identity,
  ) async {
    try {
      await tracker.videoWatched(post);
      if (_requestedIdentity == identity) _visibleIdentity = identity;
      return true;
    } on Object catch (error, stackTrace) {
      if (_visibleIdentity == identity) _visibleIdentity = null;
      if (_requestedIdentity == identity) _requestedIdentity = null;
      onWatchFailure?.call(error, stackTrace);
      return false;
    }
  }

  String _identityOf(VideoPost post) {
    final values = [...VideoWatchFingerprints.fromPost(post).values]..sort();
    return values.join();
  }
}
