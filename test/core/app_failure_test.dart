import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';

void main() {
  test('stringifies the failure message', () {
    expect(const AppFailure('Feed unavailable').toString(), 'Feed unavailable');
  });
}
