import 'package:flutter_test/flutter_test.dart';
import 'package:video_player/video_player.dart';

import '../support/video_player_controller_budget_fixture.dart';

void main() {
  testWidgets('an unproven teardown quarantines its decoder capacity', (
    tester,
  ) async {
    var disposalAttempts = 0;
    final fixture = VideoPlayerControllerBudgetFixture(
      disposer: (VideoPlayerController controller) async {
        disposalAttempts += 1;
        if (disposalAttempts == 1) {
          throw StateError('injected teardown failure');
        }
        await controller.dispose();
      },
    );
    await fixture.show(tester, ['a', 'b']);

    await fixture.show(tester, ['b', 'c']);
    expect(fixture.creations('c'), 0);
    await fixture.turn(tester);
    await tester.pump(const Duration(milliseconds: 100));

    expect(fixture.creations('c'), 0);
    expect(fixture.platform.playerCount, 2);
    expect(fixture.platform.peakPlayerCount, 2);

    await fixture.show(tester, ['c']);
    await fixture.turn(tester);
    await tester.pump(const Duration(milliseconds: 100));
    expect(fixture.creations('c'), 1);
    expect(fixture.platform.playerCount, 2);
  });
}
