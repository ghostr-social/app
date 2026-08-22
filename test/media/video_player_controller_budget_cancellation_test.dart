import 'package:flutter_test/flutter_test.dart';

import '../support/video_player_controller_budget_fixture.dart';

void main() {
  testWidgets('a removed queued surface never consumes a decoder', (
    tester,
  ) async {
    final fixture = VideoPlayerControllerBudgetFixture();
    await fixture.show(tester, ['a', 'b']);
    fixture.platform.blockDisposal();

    await fixture.show(tester, ['b', 'c']);
    await fixture.show(tester, ['b']);
    fixture.platform.releaseDisposal();
    await fixture.turn(tester);
    await fixture.show(tester, ['b', 'd']);

    expect(fixture.creations('c'), 0);
    expect(fixture.creations('d'), 1);
    expect(fixture.platform.peakPlayerCount, 2);
  });
}
