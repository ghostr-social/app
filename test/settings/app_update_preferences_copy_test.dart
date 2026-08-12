import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

void main() {
  test('app update preferences change independently', () {
    const original = AppUpdatePreferences.defaults;

    final changed = original.copyWith(
      automaticChecks: false,
      downloadPolicy: UpdateDownloadPolicy.anyNetwork,
      automaticInstall: false,
    );

    expect(changed.automaticChecks, isFalse);
    expect(changed.downloadPolicy, UpdateDownloadPolicy.anyNetwork);
    expect(changed.automaticInstall, isFalse);
    expect(
      original.copyWith(automaticChecks: false).downloadPolicy,
      UpdateDownloadPolicy.wifiOnly,
    );
    expect(
      original
          .copyWith(downloadPolicy: UpdateDownloadPolicy.manual)
          .automaticInstall,
      isTrue,
    );
    expect(original.copyWith(automaticInstall: false).automaticChecks, isTrue);
    expect(original.copyWith(), same(original));
  });
}
