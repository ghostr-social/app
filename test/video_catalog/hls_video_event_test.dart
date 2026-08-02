import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('maps a NIP-71 HLS playlist as playable video media', () {
    final event = Nip01Event(
      id: testEventId,
      pubKey:
          '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e',
      kind: 34236,
      createdAt: 1773302400,
      content: 'Adaptive relay video',
      tags: const [
        ['d', 'hls-clip'],
        [
          'imeta',
          'url https://cdn.example/video.m3u8',
          'm application/x-mpegURL',
        ],
      ],
    );

    final post = const NostrVideoEventMapper().map(event, null);

    expect(post.media.remoteUrl, 'https://cdn.example/video.m3u8');
    expect(post.media.remoteDelivery, VideoMediaDelivery.hls);
    expect(post.media.canCacheAsSingleFile, isFalse);
  });
}
