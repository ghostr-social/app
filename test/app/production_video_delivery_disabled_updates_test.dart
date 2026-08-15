import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/production_video_catalog.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_nostr_social_port.dart';
import '../support/fake_nostr_video_publisher_port.dart';
import '../support/nostr_test_values.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test('disabled video delivery omits the feed update stream', () async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    final preferences = await SharedPreferences.getInstance();
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final nostr = ProductionNostrServices(
      ProductionNostrAdapters(FakeNostrSessionPort(), FakeNostrSocialPort()),
      client,
      FakeNostrVideoPublisherPort(),
    );
    final accountScope = AccountStorageScope(() => client.publicKeyHex);
    final services = buildProductionVideoCatalog(
      ProductionVideoCatalogInputs(
        preferences: preferences,
        delivery: ProductionVideoDelivery.disabled(),
        nostr: nostr,
        accountScope: accountScope,
        watchHistory: LocalWatchHistoryRepository(
          preferences,
          database: await openTestWatchHistoryDatabase(),
          accountScope: accountScope,
        ),
      ),
    );

    expect(services.feedUpdates, isNull);
  });
}
