import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';

import '../support/app_update_status_panel_fixture.dart';

void main() {
  testWidgets('failure status keeps its safe message visible', (tester) async {
    final recorder = AppUpdateActionRecorder();

    await pumpUpdateStatus(
      tester,
      const AppUpdateFailureState('The download could not be verified.'),
      recorder.actions,
    );

    expect(find.text('Update failed'), findsOneWidget);
    expect(find.text('The download could not be verified.'), findsOneWidget);
  });
}
