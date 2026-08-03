import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/fallback_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  test('an older page comes from the primary even when it is empty', () async {
    final primary = FakeRemoteVideoSource([samplePost(id: 'primary-newest')]);
    final fallback = FakeRemoteVideoSource([samplePost(id: 'fallback-only')]);
    final source = FallbackRemoteVideoSource(
      primary: primary,
      fallback: fallback,
    );

    final posts = await source.loadRemoteFeed(
      olderThan: DateTime.utc(2026, 8, 1),
    );

    expect(posts, isEmpty);
    expect(primary.requestedOlderThan, hasLength(1));
    expect(fallback.loadCount, 0);
  });
}
