import 'package:flutter_test/flutter_test.dart';
import 'package:video_player/video_player.dart';

import '../support/video_player_controller_budget_fixture.dart';

void main() {
  testWidgets('two unproven teardowns exhaust decoder capacity visibly', (
    tester,
  ) async {
    final failedControllers = <VideoPlayerController>[];
    final fixture = VideoPlayerControllerBudgetFixture(
      disposer: (VideoPlayerController controller) async {
        failedControllers.add(controller);
        throw StateError('injected teardown failure');
      },
    );
    await fixture.show(tester, ['a', 'b']);
    await fixture.show(tester, ['b', 'c']);
    expect(fixture.creations('c'), 0);

    await fixture.show(tester, ['c']);
    await fixture.turn(tester);
    await tester.pump(const Duration(milliseconds: 100));

    expect(fixture.creations('c'), 0);
    expect(fixture.platform.playerCount, 2);
    expect(fixture.platform.peakPlayerCount, 2);
    expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);
    for (final controller in failedControllers) {
      await controller.pause();
    }
  });
}
