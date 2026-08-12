import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';

void main() {
  testWidgets('current status says Ghostr is up to date', (tester) async {
    final semantics = tester.ensureSemantics();
    final recorder = AppUpdateActionRecorder();

    await pumpUpdateStatus(
      tester,
      const AppUpdateCurrentState(),
      recorder.actions,
    );

    expect(find.text('Ghostr is up to date.'), findsOneWidget);
    expect(
      find.bySemanticsLabel(RegExp('Ghostr is up to date\\.')),
      findsOneWidget,
    );
    semantics.dispose();
  });
}
