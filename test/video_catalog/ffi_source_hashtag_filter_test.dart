import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('keeps only native videos carrying a requested hashtag', () async {
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => [],
      loader: () async => [
        ffiVideo(
          id: 'tagged',
          user: const FfiUserData(npub: 'npub1alice', name: 'Alice'),
          options: const FfiVideoFixtureOptions(localPath: '/cache/tagged.mp4'),
          event: _event(
            eventId: testEventId,
            identifier: 'tagged',
            content: 'Skate #tag run',
          ),
        ),
        ffiVideo(
          id: 'plain',
          user: const FfiUserData(npub: 'npub1bob', name: 'Bob'),
          options: const FfiVideoFixtureOptions(localPath: '/cache/plain.mp4'),
          event: _event(
            eventId: secondTestEventId,
            identifier: 'plain',
            content: 'Street clip',
          ),
        ),
      ],
    );

    final posts = await source.loadRemoteFeed(hashtags: {'tag'});

    expect(posts.map((post) => post.id.value), [testEventId]);
  });
}

FfiNostrEventIdentity _event({
  required String eventId,
  required String identifier,
  required String content,
}) {
  return FfiNostrEventIdentity(
    eventId: eventId,
    authorPublicKeyHex: testCreatorPublicKey,
    kind: BigInt.from(34235),
    identifier: identifier,
    createdAt: BigInt.from(1785628800),
    content: content,
    hashtags: const <String>[],
  );
}
