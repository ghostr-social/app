import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/startup_gate.dart';

void main() {
  testWidgets('announces application startup while dependencies load',
      (tester) async {
    final dependencies = Completer<AppDependencies>();

    await tester.pumpWidget(StartupGate(
      loadDependencies: () => dependencies.future,
    ));

    expect(find.bySemanticsLabel('Starting Ghostr'), findsOneWidget);
  });
}
