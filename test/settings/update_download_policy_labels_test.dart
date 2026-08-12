import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

void main() {
  test('download policies have user-facing labels', () {
    expect(UpdateDownloadPolicy.manual.label, 'Off');
    expect(UpdateDownloadPolicy.wifiOnly.label, 'Wi-Fi only');
    expect(UpdateDownloadPolicy.anyNetwork.label, 'Wi-Fi or mobile data');
  });
}
