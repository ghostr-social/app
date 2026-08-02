import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/deferred_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  test('builds its remote source only when first loaded and reuses it',
      () async {
    var buildCount = 0;
    final posts = [samplePost()];
    final source = DeferredRemoteVideoSource(() async {
      buildCount += 1;
      return FakeRemoteVideoSource(posts);
    });

    expect(buildCount, 0);
    expect(await source.loadRemoteFeed(), same(posts));
    expect(await source.loadRemoteFeed(), same(posts));
    expect(buildCount, 1);
  });
}
