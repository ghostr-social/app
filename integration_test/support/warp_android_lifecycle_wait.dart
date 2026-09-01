import 'package:flutter_test/flutter_test.dart';

Future<void> waitForAndroidLifecycleEvidence(
  WidgetTester tester,
  bool Function() condition,
) async {
  final watch = Stopwatch()..start();
  while (!condition() && watch.elapsed < const Duration(seconds: 15)) {
    await tester.pump(const Duration(milliseconds: 50));
    await Future<void>.delayed(const Duration(milliseconds: 20));
  }
  expect(condition(), isTrue);
}
