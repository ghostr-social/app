import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/device_video_frame_evidence.dart';
import 'support/warp_feed_playback_journey.dart';

const _integrationTestChannel = MethodChannel(
  'plugins.flutter.io/integration_test',
);

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('signed feed visibly renders changing video frames', (
    tester,
  ) async {
    final journey = await WarpFeedPlaybackJourney.start();
    addTearDown(journey.close);
    await tester.pumpWidget(journey.app);
    journey.load();
    await journey.waitForCaption(tester, 0);
    final focus = await journey.waitForPublishedFocus(tester, 0);
    await journey.waitForFirstFrame(tester, focus);
    await journey.waitForPlaying(tester, focus);

    await binding.convertFlutterSurfaceToImage();
    await tester.pump();
    final first = await binding.takeScreenshot('warp-visible-first');
    // The binding has no public revert API; restore the real surface so the
    // video texture advances, then reconvert it before the registered teardown.
    await _integrationTestChannel.invokeMethod<void>('revertFlutterImage');
    await journey.pumpFor(tester, const Duration(seconds: 1));
    await _integrationTestChannel.invokeMethod<void>(
      'convertFlutterSurfaceToImage',
    );
    await tester.pump();
    final second = await binding.takeScreenshot('warp-visible-second');
    final evidence = await DeviceVideoFrameEvidence.compare(first, second);

    expect(evidence.chromaticRatio, greaterThan(0.25));
    expect(evidence.changedRatio, greaterThan(0.05));
    expect(find.text('Video unavailable'), findsNothing);
  });
}
