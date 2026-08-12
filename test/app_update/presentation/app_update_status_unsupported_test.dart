import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';

void main() {
  testWidgets('unsupported status explains that updates are unavailable', (
    tester,
  ) async {
    final recorder = AppUpdateActionRecorder();

    await pumpUpdateStatus(
      tester,
      const AppUpdateUnsupportedState('No compatible Android APK was found.'),
      recorder.actions,
    );

    expect(find.text('Updates unavailable'), findsOneWidget);
    expect(find.text('No compatible Android APK was found.'), findsOneWidget);
  });
}
