import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('applies creator and text scopes to native video inventory', () async {
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => [
        nostrVideoPost(const NostrVideoPostFixture(
          eventId: testEventId,
          mediaId: 'alpha',
          creator: NostrCreatorFixture(npub: 'npub1alice', name: 'Alice'),
          text: NostrVideoTextFixture(
            caption: 'Relay dance',
            songName: 'Quiet song',
          ),
        )),
        nostrVideoPost(const NostrVideoPostFixture(
          eventId: secondTestEventId,
          mediaId: 'beta',
          creator: NostrCreatorFixture(npub: 'npub1bob', name: 'Bob'),
          text: NostrVideoTextFixture(
            caption: 'Second clip',
            songName: 'Loud anthem',
          ),
        )),
      ],
      loader: () async => [
        ffiVideo(
          id: 'alpha',
          user: const FfiUserData(npub: 'npub1alice', name: 'Alice'),
          options: const FfiVideoFixtureOptions(
            title: 'Relay dance',
            songName: 'Quiet song',
          ),
          event: ffiNostrEvent(identifier: 'alpha'),
        ),
        ffiVideo(
          id: 'beta',
          user: const FfiUserData(npub: 'npub1bob', name: 'Bob'),
          options: const FfiVideoFixtureOptions(
            title: 'Second clip',
            songName: 'Loud anthem',
          ),
          event: ffiNostrEvent(
            eventId: secondTestEventId,
            identifier: 'beta',
          ),
        ),
      ],
    );

    final creator = await source.loadRemoteFeed(
      creatorIds: {ProfileId.parse('npub1alice')},
    );
    final caption = await source.loadRemoteFeed(searchQuery: ' relay DANCE ');
    final song = await source.loadRemoteFeed(searchQuery: 'loud anthem');

    expect(creator.single.id.value, testEventId);
    expect(caption.single.id.value, testEventId);
    expect(song.single.id.value, secondTestEventId);
  });
}
