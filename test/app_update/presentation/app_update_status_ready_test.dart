import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';

void main() {
  testWidgets('ready status offers installation', (tester) async {
    final recorder = AppUpdateActionRecorder();

    await pumpUpdateStatus(
      tester,
      AppUpdateReadyState(sampleVerifiedUpdatePackage()),
      recorder.actions,
    );

    expect(find.text('The update is ready to install.'), findsOneWidget);
    await tester.tap(find.widgetWithText(FilledButton, 'Install update'));
    expect(recorder.calls, <String>['install']);
  });
}
