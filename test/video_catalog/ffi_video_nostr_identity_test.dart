import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('only exposes native media backed by a canonical Nostr event', () async {
    final reference = NostrEventReference(
      eventId: NostrEventId.parse(testEventId),
      authorPublicKeyHex: NostrPublicKeyHex.parse(testCreatorPublicKey),
      kind: NostrEventKind.parse(34235),
      identifier: NostrEventIdentifier.parse('dance'),
    );
    final canonical = _canonicalPost(reference);
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => [canonical],
      loader: () async => [
        ffiVideo(
          id: 'matched',
          user: const FfiUserData(),
          options: const FfiVideoFixtureOptions(localPath: '/v.mp4'),
        ),
        ffiVideo(
          id: 'anonymous',
          user: const FfiUserData(),
          event: ffiNostrEvent(eventId: 'not-an-event-id'),
        ),
      ],
    );

    final posts = await source.loadRemoteFeed();

    expect(posts, hasLength(1));
    expect(posts.single.id, canonical.id);
    expect(posts.single.nostrReference, same(reference));
    expect(posts.single.media.localPath, isNull);
    expect(
      posts.single.media.remoteUrl,
      'https://source.example/matched.mp4',
    );
  });
}

VideoPost _canonicalPost(NostrEventReference reference) {
  return VideoPost(
    identity: VideoPostIdentity(
      id: VideoPostId.parse(testEventId),
      creator: ProfileSummary(
        id: ProfileId.parse('npub1creator'),
        displayName: 'Nora',
        handle: '@npub1creator',
        avatarUrl: null,
      ),
      nostrReference: reference,
    ),
    content: VideoPostContent(
      caption: 'Relay dance',
      songName: 'Original sound',
      media: VideoMediaSource.remote('https://source.example/matched.mp4'),
      publishedAt: DateTime.utc(2026, 8, 2),
    ),
    metrics: VideoPostMetrics(
      likeCount: 4,
      commentCount: 2,
      viewerHasLiked: false,
    ),
  );
}
