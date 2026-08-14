import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/local_update_offer_history_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('ignores corrupt declined-version storage', () async {
    const key = 'ghostr.updates.lastDeclinedVersionCode';
    for (final value in <Object>['twenty-three', -1, 2100000001]) {
      SharedPreferences.setMockInitialValues({key: value});
      final preferences = await SharedPreferences.getInstance();
      final repository = LocalUpdateOfferHistoryRepository(preferences);

      expect(await repository.readLastDeclinedVersion(), isNull);
    }
  });
}
