import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:mocktail/mocktail.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/test_account_storage_scope.dart';

class _Preferences extends Mock implements SharedPreferences {}

void main() {
  test('rejects published-video storage when preferences refuse the write',
      () async {
    final preferences = _Preferences();
    when(() => preferences.setString(any(), any()))
        .thenAnswer((_) async => false);
    final store = LocalVideoStore(
      preferences,
      accountScope: testAccountStorageScope(),
    );

    await expectLater(
      store.savePublishedPosts(const []),
      throwsA(isA<AppFailure>()),
    );
  });
}
