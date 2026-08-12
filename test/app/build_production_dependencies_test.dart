import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/features/session/data/pending_first_session_repository.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/video_sharing/data/default_video_share_workflow.dart';
import 'package:ghostr/platform/media/image_picker_capabilities.dart';
import 'package:ghostr/platform/media/image_picker_media_picker.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_nostr_social_port.dart';
import '../support/fake_nostr_video_publisher_port.dart';
import '../support/fake_remote_video_source.dart';
import '../support/test_video_delivery.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('builds the real app graph from injected platform boundaries', () async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    final preferences = await SharedPreferences.getInstance();
    final nostr = ProductionNostrServices(
      ProductionNostrAdapters(FakeNostrSessionPort(), FakeNostrSocialPort()),
      FakeNostrEventClient(publicKeyHex: testViewerPublicKey),
      FakeNostrVideoPublisherPort(),
    );
    final environment = ProductionDependenciesEnvironment(
      preferencesLoader: () async => preferences,
      nostrServicesBuilder: (_) => nostr,
      videoDeliveryBuilder: (_, __) async =>
          testVideoDelivery(remoteSource: FakeRemoteVideoSource([])),
    );

    final dependencies = await buildProductionDependencies(environment);

    expect(
      dependencies.sessionRepository,
      isA<PendingFirstSessionRepository>(),
    );
    expect(dependencies.accountProvisioningRepository, isNotNull);
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
    expect(dependencies.videoShareWorkflow, isA<DefaultVideoShareWorkflow>());
  });
}
