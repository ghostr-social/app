import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('maps the Apple HLS MIME type as adaptive video media', () {
    final event = Nip01Event(
      id: testEventId,
      pubKey:
          '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e',
      kind: 34236,
      content: 'Apple adaptive video',
      tags: const [
        ['d', 'apple-hls'],
        [
          'imeta',
          'url https://cdn.example/apple.m3u8',
          'm application/vnd.apple.mpegurl',
        ],
      ],
    );

    final media = const NostrVideoEventMapper().map(event, null).media;

    expect(media.remoteUrl, 'https://cdn.example/apple.m3u8');
    expect(media.remoteDelivery, VideoMediaDelivery.hls);
  });
}
