import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:video_player/video_player.dart';

import '../support/video_player_controller_budget_fixture.dart';

void main() {
  testWidgets('two hung teardowns exhaust decoder capacity visibly', (
    tester,
  ) async {
    final never = Completer<void>().future;
    final retiring = <VideoPlayerController>[];
    final fixture = VideoPlayerControllerBudgetFixture(
      disposer: (VideoPlayerController controller) {
        retiring.add(controller);
        return never;
      },
    );
    await fixture.show(tester, ['a', 'b']);
    await fixture.show(tester, ['b', 'c']);
    await fixture.show(tester, ['c']);

    await tester.pump(playbackControllerTeardownTimeout);
    await fixture.turn(tester);
    await tester.pump(const Duration(milliseconds: 100));

    expect(fixture.creations('c'), 0);
    expect(fixture.platform.playerCount, 2);
    expect(fixture.platform.peakPlayerCount, 2);
    expect(fixture.releaseCount, 0);
    expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);

    await fixture.show(tester, ['d']);
    expect(fixture.creations('d'), 0);
    expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);
    for (final controller in retiring) {
      await controller.pause();
    }
  });
}
