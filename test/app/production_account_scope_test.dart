import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_nostr_social_port.dart';
import '../support/fake_nostr_video_publisher_port.dart';
import '../support/fake_remote_video_source.dart';
import '../support/fake_video_inventory.dart';
import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('production stores follow active account switches', () async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final social =
        FakeNostrSocialPort(activeAccount: () => client.publicKeyHex);
    final nostr = ProductionNostrServices(
      MockNdk(),
      ProductionNostrAdapters(
        FakeNostrSessionPort(),
        social,
      ),
      client,
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
    final activity = sampleActivity();
    final post = await dependencies.videoCatalogServices.publishing.publish(
      session: sampleSession(),
      media: sampleMedia(),
      caption: 'First account post',
    );
    await dependencies.activityRepository.record(activity);

    client.publicKeyHex = NostrPublicKeyHex.parse(testAuthorPublicKey);
    expect(
      await dependencies.videoCatalogServices.feed.loadFeed(FeedKind.forYou),
      isEmpty,
    );
    expect(await dependencies.activityRepository.load(), isEmpty);

    client.publicKeyHex = NostrPublicKeyHex.parse(testViewerPublicKey);
    final restoredPosts =
        await dependencies.videoCatalogServices.feed.loadFeed(FeedKind.forYou);
    expect(restoredPosts.single.id, post.post.id);
    expect(
        (await dependencies.activityRepository.load()).single.id, activity.id);
  });
}
