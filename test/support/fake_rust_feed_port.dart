import 'dart:async';

import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

/// A scripted Rust feed port: emits the configured updates on
/// subscription and records every call the source makes.
class FakeRustFeedPort implements RustFeedPort {
  FakeRustFeedPort({this.updates = const [], this.moreAvailable = true});

  List<FfiFeedUpdate> updates;
  bool moreAvailable;
  bool closeStreamAfterUpdates = true;
  Object? openError;
  Object? streamError;
  String feedId = '7';

  final List<FfiFeedSpec> openedSpecs = <FfiFeedSpec>[];
  final List<BigInt?> loadMoreCursors = <BigInt?>[];
  final List<String> closedFeedIds = <String>[];

  @override
  Future<String> openFeed(FfiFeedSpec spec) async {
    openedSpecs.add(spec);
    if (openError case final error?) throw error;
    return feedId;
  }

  @override
  Stream<FfiFeedUpdate> feedUpdates(String feedId) async* {
    for (final update in updates) {
      yield update;
    }
    if (streamError case final error?) throw error;
    if (!closeStreamAfterUpdates) await Completer<void>().future;
  }

  @override
  Future<bool> loadMore(String feedId, {BigInt? olderThanSecs}) async {
    loadMoreCursors.add(olderThanSecs);
    return moreAvailable;
  }

  @override
  Future<void> closeFeed(String feedId) async {
    closedFeedIds.add(feedId);
  }
}
