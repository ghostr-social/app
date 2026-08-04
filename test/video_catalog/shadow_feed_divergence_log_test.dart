import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/shadow_compare_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  test('logs the divergence between the ndk truth and the rust shadow',
      () async {
    final logged = <String>[];
    final source = ShadowCompareRemoteVideoSource(
      primary: FakeRemoteVideoSource([samplePost(id: 'a'), samplePost(id: 'b')]),
      shadow: FakeRemoteVideoSource([samplePost(id: 'b')]),
      logger: logged.add,
    );

    await source.loadRemoteFeed(searchQuery: 'ghost');
    await pumpEventQueue();

    expect(logged.single, contains('search:ghost'));
    expect(logged.single, contains('missing=[a]'));
  });

  test('logs a shadow failure without surfacing it', () async {
    final logged = <String>[];
    final source = ShadowCompareRemoteVideoSource(
      primary: FakeRemoteVideoSource([samplePost(id: 'a')]),
      shadow: FakeRemoteVideoSource([])
        ..failure = const AppFailure('rust feed down'),
      logger: logged.add,
    );

    await source.loadRemoteFeed();
    await pumpEventQueue();

    expect(logged.single, contains('Shadow feed failed'));
  });

  test('stays silent when both pipelines agree', () async {
    final posts = [samplePost(id: 'a'), samplePost(id: 'b')];
    final logged = <String>[];
    final source = ShadowCompareRemoteVideoSource(
      primary: FakeRemoteVideoSource(posts),
      shadow: FakeRemoteVideoSource([...posts]),
      logger: logged.add,
    );

    await source.loadRemoteFeed();
    await pumpEventQueue();

    expect(logged, isEmpty);
  });
}
