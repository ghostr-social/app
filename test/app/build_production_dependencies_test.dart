import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/platform/media/image_picker_capabilities.dart';
import 'package:ghostr/platform/media/image_picker_media_picker.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_nostr_social_port.dart';
import '../support/fake_nostr_video_publisher_port.dart';
import '../support/fake_remote_video_source.dart';
import '../support/fake_video_inventory.dart';
import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('builds the real app graph from injected platform boundaries', () async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    final preferences = await SharedPreferences.getInstance();
    final nostr = ProductionNostrServices(
      MockNdk(),
      ProductionNostrAdapters(
        FakeNostrSessionPort(),
        FakeNostrSocialPort(),
      ),
      FakeNostrEventClient(publicKeyHex: testViewerPublicKey),
      FakeNostrVideoPublisherPort(),
    );
    final environment = ProductionDependenciesEnvironment(
      preferencesLoader: () async => preferences,
      nostrServicesBuilder: (_) => nostr,
      videoDeliveryBuilder: (_, __) async => ProductionVideoDelivery(
        FakeVideoInventory(),
        FakeRemoteVideoSource([]),
      ),
    );

    final dependencies = await buildProductionDependencies(environment);

    expect(dependencies.sessionRepository, isA<SecureSessionRepository>());
    expect(
      dependencies.appSettingsRepository,
      isA<LocalAppSettingsRepository>(),
    );
    expect(dependencies.videoCatalogServices.feed, isNotNull);
    expect(dependencies.activityRepository, isNotNull);
    expect(dependencies.mediaPickerPort, isA<ImagePickerMediaPicker>());
    expect(
      dependencies.mediaPickerPort.capabilities,
      currentImagePickerCapabilities(),
    );
  });
}
