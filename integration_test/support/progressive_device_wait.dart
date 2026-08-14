import 'package:flutter_test/flutter_test.dart';

const progressiveDeviceConditionTimeout = Duration(seconds: 10);

Future<void> waitForDeviceCondition(
  WidgetTester tester,
  bool Function() condition, {
  Duration timeout = progressiveDeviceConditionTimeout,
}) async {
  final watch = Stopwatch()..start();
  while (!condition() && watch.elapsed < timeout) {
    await _tick(tester);
  }
  if (!condition()) {
    fail('Progressive device condition timed out after $timeout.');
  }
}

Future<void> waitForAsyncDeviceCondition(
  WidgetTester tester,
  Future<bool> Function() condition, {
  Duration timeout = progressiveDeviceConditionTimeout,
}) async {
  final watch = Stopwatch()..start();
  while (!await condition() && watch.elapsed < timeout) {
    await _tick(tester);
  }
  if (!await condition()) {
    fail('Progressive device condition timed out after $timeout.');
  }
}

Future<void> pumpDeviceFor(WidgetTester tester, Duration duration) async {
  final watch = Stopwatch()..start();
  while (watch.elapsed < duration) {
    await _tick(tester);
  }
}

Future<void> _tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 50));
  await Future<void>.delayed(const Duration(milliseconds: 20));
}
