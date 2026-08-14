import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';

void main() {
  testWidgets('idle status stays neutral when automatic offers are off', (
    tester,
  ) async {
    final recorder = AppUpdateActionRecorder();

    await pumpUpdateStatus(
      tester,
      const AppUpdateIdleState(),
      recorder.actions,
    );

    expect(find.text('No update check has run yet.'), findsOneWidget);
    expect(recorder.calls, isEmpty);
  });
}
