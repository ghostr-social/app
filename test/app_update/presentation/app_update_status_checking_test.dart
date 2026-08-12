import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';

void main() {
  testWidgets('checking status is visible and announced as progress', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final recorder = AppUpdateActionRecorder();

    await pumpUpdateStatus(
      tester,
      const AppUpdateCheckingState(),
      recorder.actions,
    );

    expect(find.text('Checking for updates…'), findsOneWidget);
    expect(
      find.bySemanticsLabel(RegExp('Checking for updates')),
      findsOneWidget,
    );
    semantics.dispose();
  });
}
