import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_nostr_social_port.dart';
import '../support/fake_nostr_video_publisher_port.dart';
import '../support/fake_remote_video_source.dart';
import '../support/fake_video_file_downloader.dart';
import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/stub_video_gateways.dart';

void main() {
  test('uses the production bootstrap factory with an injected video boundary',
      () async {
    final root = await Directory.systemTemp.createTemp('ghostr-bootstrap-');
    addTearDown(() => root.delete(recursive: true));
    final videoEnvironment = ProductionVideoDeliveryEnvironment(
      canonicalSource: FakeRemoteVideoSource([]),
      supportDirectoryProvider: () async => root,
      downloader: FakeVideoFileDownloader({}),
      gateway: startedVideoGateway(),
    );
    final environment = ProductionDependenciesEnvironment.production(
      videoEnvironmentBuilder: (_, __) => videoEnvironment,
    );
    final services = ProductionNostrServices(
      MockNdk(),
      ProductionNostrAdapters(
        FakeNostrSessionPort(),
        FakeNostrSocialPort(),
      ),
      FakeNostrEventClient(publicKeyHex: testViewerPublicKey),
      FakeNostrVideoPublisherPort(),
    );

    final delivery = await environment.videoDeliveryBuilder(
      AppSettings.defaults(),
      services,
    );

    expect(delivery.remoteSource, isNotNull);
  });
}
