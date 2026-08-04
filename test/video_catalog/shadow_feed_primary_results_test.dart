import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/shadow_compare_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  test('serves the primary results untouched while the shadow diverges',
      () async {
    final primaryPosts = [samplePost(id: 'a'), samplePost(id: 'b')];
    final shadow = FakeRemoteVideoSource([samplePost(id: 'z')]);
    final source = ShadowCompareRemoteVideoSource(
      primary: FakeRemoteVideoSource(primaryPosts),
      shadow: shadow,
      logger: (_) {},
    );

    final posts = await source.loadRemoteFeed(searchQuery: 'ghost');
    await pumpEventQueue();

    expect(posts, same(primaryPosts));
    // The shadow ran in parallel with the very same request.
    expect(shadow.loadCount, 1);
    expect(shadow.requestedSearchQuery, 'ghost');
  });

  test('a shadow failure never reaches the caller', () async {
    final primaryPosts = [samplePost(id: 'a')];
    final source = ShadowCompareRemoteVideoSource(
      primary: FakeRemoteVideoSource(primaryPosts),
      shadow: FakeRemoteVideoSource([])
        ..failure = const AppFailure('rust feed down'),
      logger: (_) {},
    );

    expect(await source.loadRemoteFeed(), same(primaryPosts));
    await pumpEventQueue();
  });

  test('a primary failure propagates even when the shadow succeeds',
      () async {
    final source = ShadowCompareRemoteVideoSource(
      primary: FakeRemoteVideoSource([])
        ..failure = const AppFailure('relays down'),
      shadow: FakeRemoteVideoSource([samplePost(id: 'a')]),
      logger: (_) {},
    );

    await expectLater(source.loadRemoteFeed(), throwsA(isA<AppFailure>()));
    await pumpEventQueue();
  });
}
