import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/feed_pipeline_flag.dart';
import 'package:ghostr/app/production_nostr_services.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/nostr/ndk_broadcast_adapter.dart';
import 'package:ghostr/platform/nostr/rust_broadcast_adapter.dart';

import '../support/ndk_mocks.dart';

void main() {
  test('wires social writes to the transport the feed flag selects', () {
    final ndk = MockNdk();

    final shipping = buildProductionNostrServices(
      AppSettings.defaults(),
      ndkBuilder: (_) => ndk,
    );
    final migrated = buildProductionNostrServices(
      AppSettings.defaults(),
      ndkBuilder: (_) => ndk,
      feedFlag: const FeedPipelineFlag(FeedPipelineMode.rust),
    );

    expect(shipping.adapters.broadcast, isA<NdkBroadcastAdapter>());
    expect(migrated.adapters.broadcast, isA<RustBroadcastAdapter>());
  });
}
