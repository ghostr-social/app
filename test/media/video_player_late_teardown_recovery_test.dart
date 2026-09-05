import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:video_player/video_player.dart';

import '../support/video_player_controller_budget_fixture.dart';

void main() {
  testWidgets(
    'late proven teardown restores capacity and retries the visible player',
    (tester) async {
      final gate = Completer<void>();
      final fixture = VideoPlayerControllerBudgetFixture(
        disposer: (VideoPlayerController controller) async {
          await gate.future;
          await controller.dispose();
        },
      );
      await fixture.show(tester, ['a', 'b']);
      await fixture.show(tester, ['c']);
      await tester.pump(playbackControllerTeardownTimeout);
      await fixture.turn(tester);
      expect(fixture.creations('c'), 0);
      expect(fixture.releaseCount, 0);
      expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);

      gate.complete();
      await fixture.turn(tester);
      await fixture.turn(tester);
      await tester.pump(const Duration(milliseconds: 100));

      expect(fixture.creations('c'), 1);
      expect(fixture.releaseCount, 2);
      expect(fixture.platform.playerCount, 1);
      expect(fixture.platform.peakPlayerCount, lessThanOrEqualTo(2));
      expect(find.bySemanticsLabel('Video unavailable'), findsNothing);
      expect(fixture.platform.audibleOverlap, isFalse);
    },
  );
}
