import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_nostr_social_port.dart';
import '../support/fake_nostr_video_publisher_port.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('history opens before the one-shot delivery engine starts', () async {
    SharedPreferences.setMockInitialValues({});
    var deliveryStarted = false;
    final nostr = ProductionNostrServices(
      ProductionNostrAdapters(FakeNostrSessionPort(), FakeNostrSocialPort()),
      FakeNostrEventClient(publicKeyHex: testViewerPublicKey),
      FakeNostrVideoPublisherPort(),
    );
    final environment = ProductionDependenciesEnvironment(
      preferencesLoader: SharedPreferences.getInstance,
      nostrServicesBuilder: (_) => nostr,
      videoDeliveryBuilder: (_, __) async {
        deliveryStarted = true;
        throw StateError('Delivery must not start.');
      },
      watchHistoryDatabaseLoader: () async {
        throw const AppFailure('Watch history unavailable.');
      },
    );

    await expectLater(
      buildProductionDependencies(environment),
      throwsA(isA<AppFailure>()),
    );
    expect(deliveryStarted, isFalse);
  });
}
