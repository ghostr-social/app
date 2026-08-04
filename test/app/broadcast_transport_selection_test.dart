import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/broadcast_transport_selection.dart';
import 'package:ghostr/app/feed_pipeline_flag.dart';
import 'package:ghostr/features/social/domain/signed_event_broadcast_port.dart';

void main() {
  final ndk = _FakeBroadcastPort();

  test('keeps ndk writes while the rust pipeline is only shadowing', () {
    var rustBuilds = 0;
    SignedEventBroadcastPort buildRust() {
      rustBuilds += 1;
      return _FakeBroadcastPort();
    }

    for (final mode in [FeedPipelineMode.ndk, FeedPipelineMode.shadow]) {
      expect(
        selectBroadcastTransport(mode: mode, ndk: ndk, rust: buildRust),
        same(ndk),
      );
    }
    expect(rustBuilds, 0);
  });

  test('publishes through the engine once the rust pipeline serves feeds', () {
    final rust = _FakeBroadcastPort();

    final selected = selectBroadcastTransport(
      mode: FeedPipelineMode.rust,
      ndk: ndk,
      rust: () => rust,
    );

    expect(selected, same(rust));
  });
}

final class _FakeBroadcastPort implements SignedEventBroadcastPort {
  @override
  Future<void> broadcast(String signedEventJson) async {}
}
