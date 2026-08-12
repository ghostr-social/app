import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

void main() {
  test('equal update preferences behave as values', () {
    const first = AppUpdatePreferences.defaults;
    final sameValues = AppUpdatePreferences(
      automaticChecks: true,
      downloadPolicy: UpdateDownloadPolicy.wifiOnly,
      automaticInstall: true,
    );
    const differentChecks = AppUpdatePreferences(
      automaticChecks: false,
      downloadPolicy: UpdateDownloadPolicy.wifiOnly,
      automaticInstall: true,
    );
    const differentDownload = AppUpdatePreferences(
      automaticChecks: true,
      downloadPolicy: UpdateDownloadPolicy.manual,
      automaticInstall: true,
    );
    const differentInstall = AppUpdatePreferences(
      automaticChecks: true,
      downloadPolicy: UpdateDownloadPolicy.wifiOnly,
      automaticInstall: false,
    );

    expect(first == first, isTrue);
    expect(first, sameValues);
    expect(first.hashCode, sameValues.hashCode);
    expect(first, isNot(differentChecks));
    expect(first, isNot(differentDownload));
    expect(first, isNot(differentInstall));
    expect(first == Object(), isFalse);
  });
}
