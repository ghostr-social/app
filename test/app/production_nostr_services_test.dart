import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ghostr/platform/nostr/ndk_blossom_profile_image_uploader.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('reads the active local signer from the configured NDK instance', () {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getPublicKey).thenReturn(testViewerPublicKey);

    final services = buildProductionNostrServices(
      AppSettings.defaults(),
      ndkBuilder: () => ndk,
    );

    expect(services.eventClient.publicKeyHex.value, testViewerPublicKey);
    expect(
      services.profileImageUploader,
      isA<NdkBlossomProfileImageUploader>(),
    );
  });
}
