import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // The kind and `d` tag decide whether a social write addresses the
  // event or its addressable coordinate, so both cross the feed FFI.
  test('mapped rust rows carry the reference social writes address', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedUpdate(revision: 1, posts: [
        rustFeedPost(eventKind: 34235, identifier: 'clip-1'),
      ]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final post = (await source.loadRemoteFeed(searchQuery: 'ghost')).single;

    final reference = post.nostrReference;
    expect(reference?.eventId, testEventId);
    expect(reference?.authorPublicKeyHex, testCreatorPublicKey);
    expect(reference?.kind, 34235);
    expect(reference?.identifier, 'clip-1');
    expect(
      VideoInteractionTarget.fromPost(post).value,
      'a:34235:$testCreatorPublicKey:clip-1',
    );
  });

  test('plain rust rows address the event itself', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedUpdate(revision: 1, posts: [rustFeedPost(eventKind: 1)]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final post = (await source.loadRemoteFeed(searchQuery: 'ghost')).single;

    expect(post.nostrReference?.kind, 1);
    expect(post.nostrReference?.identifier, isNull);
    expect(VideoInteractionTarget.fromPost(post).value, 'e:$testEventId');
  });
}
