import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:video_player/video_player.dart';

import '../support/video_player_controller_budget_fixture.dart';

void main() {
  testWidgets('one hung teardown cannot consume both decoder permits', (
    tester,
  ) async {
    final never = Completer<void>().future;
    final fixture = VideoPlayerControllerBudgetFixture(
      disposer: (VideoPlayerController controller) => never,
      recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero]),
      autoInitialize: false,
    );
    await fixture.show(tester, ['a'], active: 'a');

    await tester.pump(playbackInitializationTimeout);
    await fixture.turn(tester);
    expect(fixture.creations('a'), 1);

    await tester.pump(const Duration(seconds: 5));
    await fixture.turn(tester);
    expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);

    await fixture.show(tester, ['b']);
    expect(fixture.creations('b'), 1);
    expect(fixture.platform.peakPlayerCount, 2);
    expect(fixture.releaseCount, 0);

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump(const Duration(seconds: 5));
    await fixture.turn(tester);
  });
}
