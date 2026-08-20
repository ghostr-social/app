import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/feed_preparation_updates.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets(
    'current playback stays visible before its exact asset is ready',
    (tester) async {
      final updates = ControlledPlaybackPreparationUpdates();
      final playback = FakeVideoPlaybackPort();
      addTearDown(updates.close);
      await tester.pumpWidget(
        feedScreenHarness(
          FakeVideoCatalogRepository(forYouFeed: [samplePost(id: 'current')]),
          options: FeedScreenHarnessOptions(
            playbackPort: playback,
            preparationUpdates: updates,
          ),
        ),
      );
      await tester.pumpAndSettle();
      updates.publish(
        PlaybackPreparationPlan(
          revision: BigInt.one,
          currentDeliveryId: PlaybackDeliveryId.parse('current'),
        ),
      );
      await tester.pump();

      expect(playback.requests, isNotEmpty);
      expect(
        playback.requests.last.media.remoteUrl,
        'https://example.com/video/current.mp4',
      );
      expect(
        find.text('https://example.com/video/current.mp4'),
        findsOneWidget,
      );
    },
  );
}
