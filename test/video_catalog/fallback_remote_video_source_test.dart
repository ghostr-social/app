import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/fallback_remote_video_source.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('uses the downloader inventory while Nostr relays are warming up',
      () async {
    final cachedPost = samplePost(id: 'cached');
    final source = FallbackRemoteVideoSource(
      primary: FakeRemoteVideoSource([]),
      fallback: FakeRemoteVideoSource([cachedPost]),
    );

    final posts = await source.loadRemoteFeed();

    expect(posts, [cachedPost]);
  });
}
