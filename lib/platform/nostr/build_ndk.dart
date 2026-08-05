import 'package:ndk/ndk.dart';

Ndk buildNdk() {
  return Ndk(
    NdkConfig(
      eventVerifier: Bip340EventVerifier(),
      cache: MemCacheManager(),
      bootstrapRelays: const [],
      engine: NdkEngine.JIT,
      logLevel: LogLevel.error,
    ),
  );
}
