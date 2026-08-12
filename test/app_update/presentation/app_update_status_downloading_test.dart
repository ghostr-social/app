import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';
import '../support/update_domain_fixture.dart';

void main() {
  testWidgets('download status exposes accessible determinate progress', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final recorder = AppUpdateActionRecorder();
    final release = sampleStableRelease();
    final state = AppUpdateDownloadingState(
      release: release,
      artifact: sampleArtifact(AndroidAbi.arm64V8a),
      bytes: 2,
      totalBytes: 4,
    );

    await pumpUpdateStatus(tester, state, recorder.actions);

    expect(find.text('Downloading Ghostr 0.0.2…'), findsOneWidget);
    expect(find.text('50%'), findsOneWidget);
    expect(find.bySemanticsLabel(RegExp('Downloading update')), findsOneWidget);
    final progress = tester.widget<LinearProgressIndicator>(
      find.byType(LinearProgressIndicator),
    );
    expect(progress.value, 0.5);
    semantics.dispose();
  });
}
