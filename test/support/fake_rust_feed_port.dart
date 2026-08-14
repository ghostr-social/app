import 'dart:async';

import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
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
  RustFeedId feedId = RustFeedId.parse('7');

  final List<FfiFeedSpec> openedSpecs = <FfiFeedSpec>[];
  final List<NostrPublicKeyHex?> capturedAccounts = <NostrPublicKeyHex?>[];
  final List<BigInt?> loadMoreCursors = <BigInt?>[];
  final List<RustFeedId> closedFeedIds = <RustFeedId>[];

  @override
  Future<RustFeedAccountSession> captureSession(
    NostrPublicKeyHex? expectedAccount,
  ) async {
    capturedAccounts.add(expectedAccount);
    return RustFeedAccountSession(
      account: expectedAccount,
      generation: RustNostrSessionGeneration.fromBridge(BigInt.zero),
    );
  }

  @override
  Future<RustFeedId> openFeed(
    FfiFeedSpec spec,
    RustFeedAccountSession session,
  ) async {
    openedSpecs.add(spec);
    if (openError case final error?) throw error;
    return feedId;
  }

  @override
  Stream<FfiFeedUpdate> feedUpdates(RustFeedId feedId) async* {
    for (final update in updates) {
      yield update;
    }
    if (streamError case final error?) throw error;
    if (!closeStreamAfterUpdates) await Completer<void>().future;
  }

  @override
  Future<bool> loadMore(RustFeedId feedId, {BigInt? olderThanSecs}) async {
    loadMoreCursors.add(olderThanSecs);
    return moreAvailable;
  }

  @override
  Future<void> closeFeed(RustFeedId feedId) async {
    closedFeedIds.add(feedId);
  }
}
