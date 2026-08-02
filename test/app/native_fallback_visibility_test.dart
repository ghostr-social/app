import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/remote_video_delivery_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  test('keeps native Nostr inventory visible when relay loading is empty',
      () async {
    final fallbackPost = samplePost(id: 'native-fallback');
    final source = buildRemoteVideoDeliverySource(
      primary: FakeRemoteVideoSource([]),
      nativeFallback: FakeRemoteVideoSource([fallbackPost]),
    );

    final posts = await source.loadRemoteFeed();

    expect(posts, [fallbackPost]);
  });
}
