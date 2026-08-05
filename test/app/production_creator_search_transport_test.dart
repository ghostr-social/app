import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/production_video_catalog.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_nostr_social_port.dart';
import '../support/fake_nostr_video_publisher_port.dart';
import '../support/fake_remote_video_source.dart';
import '../support/nostr_test_values.dart';
import '../support/test_video_delivery.dart';

void main() {
  test('production creator search reads kind-0 events through event client',
      () async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    final preferences = await SharedPreferences.getInstance();
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.add(_metadata());
    final nostr = ProductionNostrServices(
      ProductionNostrAdapters(
        FakeNostrSessionPort(),
        FakeNostrSocialPort(),
      ),
      client,
      FakeNostrVideoPublisherPort(),
    );
    final accountScope = AccountStorageScope(() => client.publicKeyHex);
    final settings = LocalAppSettingsRepository(preferences);
    final services = buildProductionVideoCatalog(ProductionVideoCatalogInputs(
      preferences: preferences,
      delivery: testVideoDelivery(remoteSource: FakeRemoteVideoSource([])),
      nostr: nostr,
      accountScope: accountScope,
      watchHistory: LocalWatchHistoryRepository(
        preferences,
        accountScope: accountScope,
      ),
      settingsRepository: settings,
    ));

    final creators = await services.search.searchCreators('alice');

    expect(creators.single.displayName, 'Alice');
    expect(client.queries.single.kinds.single.value, 0);
    expect(client.queries.single.search, 'alice');
  });
}

NostrEventRecord _metadata() {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: testEventId,
      authorPublicKeyHex: testCreatorPublicKey,
      kind: 0,
    ),
    tags: const [],
    content: '{"display_name":"Alice"}',
    createdAt: 10,
  );
}
