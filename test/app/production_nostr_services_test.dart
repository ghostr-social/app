import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_session.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';

import '../support/ndk_mocks.dart';

void main() {
  test('builds Nostr adapters around the configured NDK instance', () {
    final ndk = MockNdk();

    final services = buildProductionNostrServices(
      AppSettings.defaults(),
      ndkBuilder: (_) => ndk,
    );

    expect(services.ndk, same(ndk));
    expect(services.adapters.session, isA<NdkNostrSession>());
    expect(services.adapters.social, isA<NdkNostrSocial>());
  });
}
