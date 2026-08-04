import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/feed_pipeline_flag.dart';
import 'package:ghostr/features/video_catalog/data/shadow_compare_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  final ndk = FakeRemoteVideoSource([samplePost(id: 'ndk')]);

  test('ndk stays the default pipeline and skips building the rust source',
      () {
    var rustBuilds = 0;
    final selected = const FeedPipelineFlag().select(
      ndk: ndk,
      rust: () {
        rustBuilds += 1;
        return FakeRemoteVideoSource([]);
      },
    );

    expect(selected, same(ndk));
    expect(rustBuilds, 0);
  });

  test('rust mode serves the rust source directly', () {
    final rust = FakeRemoteVideoSource([samplePost(id: 'rust')]);
    final selected = const FeedPipelineFlag(FeedPipelineMode.rust).select(
      ndk: ndk,
      rust: () => rust,
    );

    expect(selected, same(rust));
  });

  test('shadow mode wraps ndk truth around the rust shadow', () async {
    final rust = FakeRemoteVideoSource([samplePost(id: 'rust')]);
    final selected = const FeedPipelineFlag(FeedPipelineMode.shadow).select(
      ndk: ndk,
      rust: () => rust,
    );

    expect(selected, isA<ShadowCompareRemoteVideoSource>());
    final posts = await selected.loadRemoteFeed();
    await pumpEventQueue();
    expect(posts.single.id.value, 'ndk');
    expect(rust.loadCount, 1);
  });
}
