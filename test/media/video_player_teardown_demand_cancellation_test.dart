import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:video_player/video_player.dart';

import '../support/video_player_controller_budget_fixture.dart';

void main() {
  testWidgets(
    'cancelled teardown demand can be rearmed from a fresh deadline',
    (tester) async {
      final firstDisposal = Completer<void>();
      var attempts = 0;
      final fixture = VideoPlayerControllerBudgetFixture(
        autoInitialize: false,
        recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero]),
        disposer: (VideoPlayerController controller) {
          if (attempts++ == 0) return firstDisposal.future;
          return controller.dispose();
        },
      );
      await fixture.show(tester, ['a'], active: 'a');
      await tester.pump(playbackInitializationTimeout);
      await fixture.turn(tester);

      await fixture.show(tester, const [], active: 'a');
      await tester.pump(const Duration(seconds: 4));
      await fixture.show(tester, ['b'], active: 'b');
      fixture.platform.initialize(1);
      await fixture.turn(tester);
      await fixture.show(tester, ['b', 'c'], active: 'b');
      await tester.pump(const Duration(seconds: 2));

      firstDisposal.complete();
      await fixture.turn(tester);
      await tester.pump(const Duration(milliseconds: 100));

      expect(fixture.creations('c'), 1);
      expect(fixture.releaseCount, 1);
    },
  );
}
