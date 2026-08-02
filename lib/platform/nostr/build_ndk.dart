import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ndk/ndk.dart';

Ndk buildNdk(List<RelayUrl> relays) {
  return Ndk(
    NdkConfig(
      eventVerifier: Bip340EventVerifier(),
      cache: MemCacheManager(),
      bootstrapRelays: relays.map((relay) => relay.value).toList(),
      engine: NdkEngine.JIT,
      logLevel: LogLevel.error,
    ),
  );
}
