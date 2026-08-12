import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';

import '../support/app_update_status_panel_fixture.dart';

void main() {
  testWidgets('lost Android confirmation offers a recovery action', (
    tester,
  ) async {
    final recorder = AppUpdateActionRecorder();

    await pumpUpdateStatus(
      tester,
      installingState(UpdateInstallStatus.awaitingUserAction),
      recorder.actions,
    );

    final retry = find.widgetWithText(FilledButton, 'Open Android installer');
    expect(retry, findsOneWidget);
    await tester.tap(retry);
    expect(recorder.calls, <String>['confirm']);
  });
}
