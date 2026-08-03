import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/test_account_storage_scope.dart';

void main() {
  test('translates malformed published-video storage into a safe failure',
      () async {
    final accountScope = testAccountStorageScope();
    SharedPreferences.setMockInitialValues({
      accountScope.capture().key('ghostr.catalog.published'): '[malformed',
    });
    final store = LocalVideoStore(
      await SharedPreferences.getInstance(),
      accountScope: accountScope,
    );

    await expectLater(store.loadPublishedPosts(), throwsA(isA<AppFailure>()));
  });
}
