import 'package:mocktail/mocktail.dart';
import 'package:ndk/entities.dart' show RelayBroadcastResponse;
import 'package:ndk/ndk.dart';

class MockNdk extends Mock implements Ndk {}

class MockNdkConfig extends Mock implements NdkConfig {}

class MockCacheManager extends Mock implements CacheManager {}

class MockAccounts extends Mock implements Accounts {}

class MockEventSigner extends Mock implements EventSigner {}

class MockRequests extends Mock implements Requests {}

class MockBroadcast extends Mock implements Broadcast {}

class MockNdkBroadcastResponse extends Mock implements NdkBroadcastResponse {}

class MockFiles extends Mock implements Files {}

class MockMetadatas extends Mock implements Metadatas {}

class MockLists extends Mock implements Lists {}

class MockFollows extends Mock implements Follows {}

List<RelayBroadcastResponse> successfulRelayBroadcast() {
  return [
    RelayBroadcastResponse(
      relayUrl: 'wss://relay.example',
      broadcastSuccessful: true,
    ),
  ];
}

void registerNdkFallbackValues() {
  registerFallbackValue(Filter());
  registerFallbackValue(<Filter>[]);
  registerFallbackValue(<String>[]);
  registerFallbackValue(Duration.zero);
  registerFallbackValue(ContactList(pubKey: 'fallback', contacts: <String>[]));
  registerFallbackValue(
    Nip01Event(pubKey: 'fallback', kind: 1, tags: const [], content: ''),
  );
}

void stubEventSigner(MockEventSigner signer, String publicKey) {
  when(signer.getPublicKey).thenReturn(publicKey);
  when(signer.canSign).thenReturn(true);
  when(() => signer.sign(any())).thenAnswer((call) async {
    return (call.positionalArguments.single as Nip01Event).copyWith(sig: 'sig');
  });
}
