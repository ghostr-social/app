import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('translates malformed activity storage into an app-safe failure',
      () async {
    SharedPreferences.setMockInitialValues({
      'ghostr.activity.items': '{malformed',
    });
    final repository = LocalActivityRepository(
      await SharedPreferences.getInstance(),
    );

    await expectLater(repository.load(), throwsA(isA<AppFailure>()));
  });
}
