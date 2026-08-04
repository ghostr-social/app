import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_nostr_social_port.dart';
import '../support/fake_nostr_video_publisher_port.dart';
import '../support/fake_remote_video_source.dart';
import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/stub_video_gateways.dart';

void main() {
  // Dependencies are composed once at startup, before any session is
  // restored, so the bootstrap must hand down a live reader instead of
  // whoever happened to be signed in then.
  test('hands the video environment a viewer that follows the account',
      () async {
    final root = await Directory.systemTemp.createTemp('ghostr-viewer-boot-');
    addTearDown(() => root.delete(recursive: true));
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    RustFeedViewer? handed;
    final bootstrap = ProductionDependenciesEnvironment.production(
      videoEnvironmentBuilder: (_, __, viewer) {
        handed = viewer;
        return ProductionVideoDeliveryEnvironment(
          canonicalSource: FakeRemoteVideoSource([]),
          supportDirectoryProvider: () async => root,
          gateway: startedVideoGateway(),
          viewer: viewer,
        );
      },
    );

    await bootstrap.videoDeliveryBuilder(
      AppSettings.defaults(),
      _services(client),
    );

    expect(handed?.call(), testViewerPublicKey);
    client.publicKeyHex = NostrPublicKeyHex.parse(testCreatorPublicKey);
    expect(handed?.call(), testCreatorPublicKey);
  });
}

ProductionNostrServices _services(FakeNostrEventClient client) {
  return ProductionNostrServices(
    MockNdk(),
    ProductionNostrAdapters(FakeNostrSessionPort(), FakeNostrSocialPort()),
    client,
    FakeNostrVideoPublisherPort(),
  );
}
