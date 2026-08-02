import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('translates malformed published-video storage into a safe failure',
      () async {
    SharedPreferences.setMockInitialValues({
      'ghostr.catalog.published': '[malformed',
    });
    final store = LocalVideoStore(await SharedPreferences.getInstance());

    await expectLater(store.loadPublishedPosts(), throwsA(isA<AppFailure>()));
  });
}
