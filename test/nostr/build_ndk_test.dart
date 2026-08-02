import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/build_ndk.dart';
import 'package:ndk/ndk.dart';

void main() {
  test('builds NDK with the configured bootstrap relays and verifier',
      () async {
    final ndk = buildNdk([
      RelayUrl.parse('wss://one.example'),
      RelayUrl.parse('wss://two.example'),
    ]);
    addTearDown(ndk.destroy);

    expect(ndk.config.bootstrapRelays, [
      'wss://one.example',
      'wss://two.example',
    ]);
    expect(ndk.config.eventVerifier, isA<Bip340EventVerifier>());
    expect(ndk.config.engine, NdkEngine.JIT);
    expect(ndk.config.logLevel, LogLevel.error);
  });
}
