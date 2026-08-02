import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/startup_gate.dart';
import 'package:ghostr/main.dart' as app;

void main() {
  testWidgets('launches the tested production startup root', (tester) async {
    Widget? root;

    await app.launchGhostr((widget) => root = widget);

    expect(root, isA<StartupGate>());
  });
}
