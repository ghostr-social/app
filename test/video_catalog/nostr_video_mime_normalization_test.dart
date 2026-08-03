import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('normalizes casing and whitespace in Nostr video MIME fields', () {
    final scenarios = [
      (
        mime: '  VIDEO/MP4 ',
        delivery: VideoMediaDelivery.progressive,
      ),
      (
        mime: ' Application/Vnd.Apple.MpegURL  ',
        delivery: VideoMediaDelivery.hls,
      ),
    ];

    for (final scenario in scenarios) {
      final event = Nip01Event(
        id: testEventId,
        pubKey: testViewerPublicKey,
        kind: 22,
        createdAt: 1773302400,
        content: 'Normalized MIME',
        tags: [
          [
            'imeta',
            'url https://media.test/video',
            'm ${scenario.mime}',
          ],
        ],
      );

      final media = const NostrVideoEventMapper().map(event, null).media;

      expect(media.remoteDelivery, scenario.delivery);
    }
  });
}
