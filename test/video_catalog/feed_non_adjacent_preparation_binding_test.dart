import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/feed_preparation_fixture.dart';
import '../support/recording_video_playback_port.dart';

void main() {
  testWidgets('deep preparation is never bound to an unready adjacent post', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture();
    final playback = RecordingVideoPlaybackPort();
    addTearDown(fixture.updates.close);
    await fixture.pump(tester, playbackPort: playback);

    fixture.publishWindow(1, 'p0', ['p2']);
    await fixture.settle(tester);

    final deep = playback.requests
        .whereType<PreparedProgressiveVideoPlaybackRequest>()
        .where(
          (request) => request.prepared.authority.deliveryId.value == 'p2',
        );
    expect(deep, isEmpty);
  });
}
