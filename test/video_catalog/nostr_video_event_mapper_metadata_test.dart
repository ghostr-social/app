import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('threads imeta size and duration onto the mapped media source', () {
    final event = Nip01Event(
      id: testEventId,
      pubKey: testCreatorPublicKey,
      kind: 22,
      createdAt: 1773302400,
      content: 'A real Nostr short',
      tags: const [
        [
          'imeta',
          'url https://cdn.example/video.mp4',
          'm video/mp4',
          'size 123456',
          'duration 42',
        ],
      ],
    );

    final post = const NostrVideoEventMapper().map(event, null);

    expect(
      post.media.mediaMetadata,
      const VideoMediaMetadata(sizeBytes: 123456, durationMs: 42000),
    );
  });
}
