import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';
import '../support/update_domain_fixture.dart';

void main() {
  testWidgets('Settings shows an offer action error with a retry action', (
    tester,
  ) async {
    final recorder = AppUpdateActionRecorder();
    final release = sampleStableRelease();
    await pumpUpdateStatus(
      tester,
      AppUpdateOfferedState(
        release,
        release.artifactFor(AndroidAbi.arm64V8a)!,
        message: 'Connect to the internet to download the update.',
      ),
      recorder.actions,
    );

    const message = 'Connect to the internet to download the update.';
    expect(find.text(message), findsOneWidget);
    expect(
      find.byWidgetPredicate(
        (widget) =>
            widget is Semantics &&
            widget.properties.liveRegion == true &&
            widget.properties.label == message,
      ),
      findsOneWidget,
    );
    final retry = find.widgetWithText(FilledButton, 'Download update');
    expect(tester.widget<FilledButton>(retry).onPressed, isNotNull);
    await tester.tap(retry);
    expect(recorder.calls, ['download']);
  });
}
