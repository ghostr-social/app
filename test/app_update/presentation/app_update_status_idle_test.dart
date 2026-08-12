import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';

void main() {
  testWidgets('idle status explains that update checks are ready', (
    tester,
  ) async {
    final recorder = AppUpdateActionRecorder();

    await pumpUpdateStatus(
      tester,
      const AppUpdateIdleState(),
      recorder.actions,
    );

    expect(find.text('Automatic update checks are ready.'), findsOneWidget);
    expect(recorder.calls, isEmpty);
  });
}
