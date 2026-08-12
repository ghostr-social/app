import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';
import '../support/update_domain_fixture.dart';

void main() {
  testWidgets('Wi-Fi gated status explains why download is waiting', (
    tester,
  ) async {
    final recorder = AppUpdateActionRecorder();
    final release = sampleStableRelease();

    await pumpUpdateStatus(
      tester,
      AppUpdateWaitingForWifiState(
        release,
        sampleArtifact(AndroidAbi.arm64V8a),
      ),
      recorder.actions,
    );

    expect(find.text('Waiting for Wi-Fi.'), findsOneWidget);
    expect(find.textContaining('0.0.2'), findsOneWidget);
  });
}
