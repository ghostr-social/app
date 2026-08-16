import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/sample_data.dart';

void main() {
  test('a delivery-changing revision cannot replace pinned media', () {
    const url = 'https://example.com/video/coordinate.mp4';
    final held = samplePost(
      id: 'coordinate',
    ).withMedia(VideoMediaSource.remote(url));
    final changed = samplePost(
      id: 'coordinate',
    ).withMedia(VideoMediaSource.remote(url, delivery: VideoMediaDelivery.hls));

    final refreshed = FeedRoster([
      held,
    ]).resynced([changed], eligible: [changed]);

    expect(
      refreshed.active.media.remoteDelivery,
      VideoMediaDelivery.progressive,
    );
  });
}
