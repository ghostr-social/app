import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';
import '../support/update_domain_fixture.dart';

void main() {
  testWidgets('Settings disables download while an offer action is pending', (
    tester,
  ) async {
    final recorder = AppUpdateActionRecorder();
    final release = sampleStableRelease();
    await pumpUpdateStatus(
      tester,
      AppUpdateOfferedState(
        release,
        release.artifactFor(AndroidAbi.arm64V8a)!,
        pendingAction: AppUpdateOfferAction.accepting,
      ),
      recorder.actions,
    );

    expect(find.bySemanticsLabel(RegExp('Starting update')), findsOneWidget);
    final button = tester.widget<FilledButton>(
      find.widgetWithText(FilledButton, 'Download update'),
    );
    expect(button.onPressed, isNull);
  });
}
