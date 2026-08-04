import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/feed_pipeline_flag.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';
import '../support/stub_video_gateways.dart';

void main() {
  test('scopes the rust feed source to the environment viewer', () async {
    final root = await Directory.systemTemp.createTemp('ghostr-rust-viewer-');
    addTearDown(() => root.delete(recursive: true));
    RustFeedViewer? scoped;
    final environment = ProductionVideoDeliveryEnvironment(
      canonicalSource: FakeRemoteVideoSource([samplePost(id: 'ndk-post')]),
      supportDirectoryProvider: () async => root,
      gateway: startedVideoGateway(),
      feedFlag: const FeedPipelineFlag(FeedPipelineMode.rust),
      viewer: () => NostrPublicKeyHex.parse(testViewerPublicKey),
      rustFeedSourceBuilder: (viewer) {
        scoped = viewer;
        return FakeRemoteVideoSource([samplePost(id: 'rust-post')]);
      },
    );

    await buildProductionVideoDelivery(AppSettings.defaults(), environment);

    expect(scoped?.call(), testViewerPublicKey);
  });
}
