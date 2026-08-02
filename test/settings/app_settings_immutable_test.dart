import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

void main() {
  test('does not expose mutable relay or upload-server collections', () {
    final settings = AppSettings.defaults();

    expect(() => settings.relays.clear(), throwsUnsupportedError);
    expect(() => settings.blossomServers.clear(), throwsUnsupportedError);
  });
}
