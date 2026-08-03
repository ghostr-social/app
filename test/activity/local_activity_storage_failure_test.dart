import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/test_account_storage_scope.dart';

void main() {
  test('translates malformed activity storage into an app-safe failure',
      () async {
    final accountScope = testAccountStorageScope();
    SharedPreferences.setMockInitialValues({
      accountScope.capture().key('ghostr.activity.items'): '{malformed',
    });
    final repository = LocalActivityRepository(
      await SharedPreferences.getInstance(),
      accountScope: accountScope,
    );

    await expectLater(repository.load(), throwsA(isA<AppFailure>()));
  });
}
