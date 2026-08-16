import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/sample_data.dart';

void main() {
  test('an eligible revision cannot replace pinned media', () {
    final source = samplePost(id: 'coordinate').media;
    final held = samplePost(
      id: 'coordinate',
    ).withMedia(VideoMediaSource.withExpectedSha256(source, 'a' * 64));
    final changed = samplePost(
      id: 'coordinate',
    ).withMedia(VideoMediaSource.withExpectedSha256(source, 'b' * 64));

    final refreshed = FeedRoster([
      held,
    ]).resynced([changed], eligible: [changed]);

    expect(refreshed.active.id, held.id);
    expect(refreshed.active.media.remoteUrl, held.media.remoteUrl);
    expect(refreshed.active.media.expectedSha256, held.media.expectedSha256);
  });
}
