import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/remote_video_delivery_source.dart';

import '../support/fake_remote_video_source.dart';

void main() {
  // Plan §4 step 10 accepted regression: the viewer-blind native fallback
  // is retired, so an empty relay outcome is served as-is.
  test('serves the relay outcome as-is without a native fallback', () async {
    final source = buildRemoteVideoDeliverySource(
      primary: FakeRemoteVideoSource([]),
    );

    final posts = await source.loadRemoteFeed();

    expect(posts, isEmpty);
  });
}
