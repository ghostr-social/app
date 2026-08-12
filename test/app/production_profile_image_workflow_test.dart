import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_nostr_social_port.dart';
import '../support/fake_nostr_video_publisher_port.dart';
import '../support/fake_profile_image_services.dart';
import '../support/fake_remote_video_source.dart';
import '../support/nostr_test_values.dart';
import '../support/test_video_delivery.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'production composition uses its configured profile image uploader',
    () async {
      SharedPreferences.setMockInitialValues({});
      final preferences = await SharedPreferences.getInstance();
      final uploader = FakeProfileImageUploader();
      final nostr = ProductionNostrServices(
        ProductionNostrAdapters(FakeNostrSessionPort(), FakeNostrSocialPort()),
        FakeNostrEventClient(publicKeyHex: testViewerPublicKey),
        FakeNostrVideoPublisherPort(),
        uploader,
      );
      final dependencies = composeProductionDependencies(
        ProductionDependencyInputs(
          preferences: preferences,
          settingsRepository: LocalAppSettingsRepository(preferences),
          nostr: nostr,
          delivery: testVideoDelivery(remoteSource: FakeRemoteVideoSource([])),
          appUpdateRuntime: null,
        ),
      );
      final metadata = ProfileMetadata.parse(
        displayName: 'Nora',
        handle: 'nora',
      );
      final selected = sampleProfileImage();

      final resolved = await dependencies.profileImageWorkflow.resolve(
        metadata,
        selected,
      );

      expect(uploader.uploaded, same(selected));
      expect(resolved.pictureUrl?.value, uploader.url);
    },
  );
}
