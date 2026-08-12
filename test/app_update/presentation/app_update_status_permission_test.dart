import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';

void main() {
  testWidgets('permission status offers settings and retry actions', (
    tester,
  ) async {
    final recorder = AppUpdateActionRecorder();
    final state = AppUpdatePermissionRequiredState(
      sampleVerifiedUpdatePackage(),
      UpdateInstallMode.confirmationRequired,
    );

    await pumpUpdateStatus(tester, state, recorder.actions);

    expect(find.text('Allow Ghostr to install updates.'), findsOneWidget);
    await tester.tap(find.widgetWithText(FilledButton, 'Allow updates'));
    await tester.tap(find.widgetWithText(TextButton, 'Retry installation'));
    expect(recorder.calls, <String>['permission', 'retry']);
  });
}
