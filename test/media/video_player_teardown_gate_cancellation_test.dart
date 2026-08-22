import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:video_player/video_player.dart';

import '../support/video_player_controller_budget_fixture.dart';

void main() {
  testWidgets('covering a teardown wait preserves later recovery', (
    tester,
  ) async {
    final firstDisposal = Completer<void>();
    var disposals = 0;
    final fixture = VideoPlayerControllerBudgetFixture(
      recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero]),
      disposer: (VideoPlayerController controller) {
        if (disposals++ == 0) return firstDisposal.future;
        return controller.dispose();
      },
    );
    await fixture.show(tester, ['a'], active: 'a');
    fixture.platform.fail(0);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 1));

    await fixture.show(tester, ['a'], active: 'covered');
    await fixture.turn(tester);
    firstDisposal.complete();
    await fixture.turn(tester);
    await fixture.show(tester, ['a'], active: 'a');
    await fixture.turn(tester);
    await tester.pump(const Duration(milliseconds: 100));

    expect(fixture.creations('a'), 2);
    expect(find.bySemanticsLabel('Video unavailable'), findsNothing);
  });
}
