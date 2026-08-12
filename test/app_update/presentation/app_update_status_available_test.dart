import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';
import '../support/update_domain_fixture.dart';

void main() {
  testWidgets('available status offers an accessible download action', (
    tester,
  ) async {
    final recorder = AppUpdateActionRecorder();
    final release = sampleStableRelease();

    await pumpUpdateStatus(
      tester,
      AppUpdateAvailableState(release, sampleArtifact(AndroidAbi.arm64V8a)),
      recorder.actions,
    );

    expect(find.text('Ghostr 0.0.2 is available.'), findsOneWidget);
    await tester.tap(find.widgetWithText(FilledButton, 'Download update'));
    expect(recorder.calls, <String>['download']);
  });
}
