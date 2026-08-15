import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/sample_data.dart';

void main() {
  test('ordinary resync installs only the admitted same-target revision', () {
    final held = samplePost(id: 'coordinate');
    final eligible = samplePost(id: 'coordinate', caption: 'Eligible revision');
    final rejected = samplePost(
      id: 'coordinate',
      caption: 'Rejected revision',
    ).withMedia(samplePost(id: 'watched-media').media);

    final refreshed = FeedRoster([held]).resynced(
      [eligible, rejected],
      eligible: [eligible],
      retainWatched: false,
    );

    expect(refreshed.active.caption, 'Eligible revision');
    expect(refreshed.active.media.remoteUrl, eligible.media.remoteUrl);
  });
}
