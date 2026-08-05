import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/build_ndk.dart';
import 'package:ndk/ndk.dart';

void main() {
  test('builds signing-only NDK without relay transports', () async {
    final ndk = buildNdk();
    addTearDown(ndk.destroy);

    expect(ndk.config.bootstrapRelays, isEmpty);
    expect(ndk.config.eventVerifier, isA<Bip340EventVerifier>());
    expect(ndk.config.engine, NdkEngine.JIT);
    expect(ndk.config.logLevel, LogLevel.error);
  });
}
