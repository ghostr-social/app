part of 'feed_cubit.dart';

final class _FeedUpdateState {
  _FeedUpdateState(this.retry);

  final FeedUpdateRetry retry;
  StreamSubscription<VideoFeedUpdate>? subscription;
  FeedKind? kind;
  int feed = 0;
  int listener = 0;
  int pulls = 0;
  BigInt revision = BigInt.from(-1);
  BigInt? pendingRevision;
  bool pendingAllowsEmpty = false;
  int? reloadingFeed;
}
