import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/src/rust/api/feed_control.dart' as feed_control;
import 'package:ghostr/src/rust/api/feed_types.dart';
import 'package:ghostr/src/rust/api/feed_updates_stream.dart' as feed_stream;

typedef RustFeedOpen = Future<String> Function({required FfiFeedSpec spec});
typedef RustFeedWatch = Stream<FfiFeedUpdate> Function(
    {required String feedId});
typedef RustFeedMore = Future<bool> Function(
    {required String feedId, BigInt? olderThanSecs});
typedef RustFeedClose = Future<void> Function({required String feedId});

/// The production [RustFeedPort]: forwards straight to the generated
/// flutter_rust_bridge feed functions, each injectable for tests.
final class FfiRustFeedPort implements RustFeedPort {
  const FfiRustFeedPort({
    RustFeedOpen open = feed_control.ffiOpenFeed,
    RustFeedWatch watch = feed_stream.ffiFeedUpdates,
    RustFeedMore more = feed_control.ffiLoadMore,
    RustFeedClose close = feed_control.ffiCloseFeed,
  })  : _open = open,
        _watch = watch,
        _more = more,
        _close = close;

  final RustFeedOpen _open;
  final RustFeedWatch _watch;
  final RustFeedMore _more;
  final RustFeedClose _close;

  @override
  Future<String> openFeed(FfiFeedSpec spec) => _open(spec: spec);

  @override
  Stream<FfiFeedUpdate> feedUpdates(String feedId) => _watch(feedId: feedId);

  @override
  Future<bool> loadMore(String feedId, {BigInt? olderThanSecs}) {
    return _more(feedId: feedId, olderThanSecs: olderThanSecs);
  }

  @override
  Future<void> closeFeed(String feedId) => _close(feedId: feedId);
}
