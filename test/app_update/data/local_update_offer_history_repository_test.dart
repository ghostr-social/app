import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/local_update_offer_history_repository.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('persists the highest declined update version', () async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    final preferences = await SharedPreferences.getInstance();
    final repository = LocalUpdateOfferHistoryRepository(preferences);

    expect(await repository.readLastDeclinedVersion(), isNull);
    await repository.recordDeclinedVersion(AndroidVersionCode(23));
    await repository.recordDeclinedVersion(AndroidVersionCode(22));

    expect((await repository.readLastDeclinedVersion())!.value, 23);
  });
}
