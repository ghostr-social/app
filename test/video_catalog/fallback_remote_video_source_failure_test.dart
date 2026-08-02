import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/fallback_remote_video_source.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('uses the warmed inventory when the primary source throws', () async {
    final cachedPost = samplePost(id: 'cached-after-failure');
    final primary = FakeRemoteVideoSource([])
      ..failure = const AppFailure('relay unavailable');
    final source = FallbackRemoteVideoSource(
      primary: primary,
      fallback: FakeRemoteVideoSource([cachedPost]),
    );

    final posts = await source.loadRemoteFeed(searchQuery: 'nostr');

    expect(posts, [cachedPost]);
  });
}
