import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';

import '../support/app_update_status_panel_fixture.dart';

void main() {
  testWidgets('installing status shows Android state and refreshes it', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final recorder = AppUpdateActionRecorder();
    const messages = <UpdateInstallStatus, String>{
      UpdateInstallStatus.pending: 'Preparing the Android installer…',
      UpdateInstallStatus.awaitingUserAction: 'Confirm the update in Android.',
      UpdateInstallStatus.succeeded: 'Android installed the update.',
      UpdateInstallStatus.failed: 'Android could not install the update.',
    };

    for (final entry in messages.entries) {
      await pumpUpdateStatus(
        tester,
        installingState(entry.key),
        recorder.actions,
      );
      expect(find.text(entry.value), findsOneWidget);
    }

    expect(find.bySemanticsLabel(RegExp('Installing update')), findsOneWidget);
    await tester.tap(
      find.widgetWithText(OutlinedButton, 'Refresh install status'),
    );
    expect(recorder.calls, <String>['refresh']);
    semantics.dispose();
  });
}
