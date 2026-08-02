import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

class MockNdk extends Mock implements Ndk {}

class MockAccounts extends Mock implements Accounts {}

class MockRequests extends Mock implements Requests {}

class MockBroadcast extends Mock implements Broadcast {}

class MockFiles extends Mock implements Files {}

class MockMetadatas extends Mock implements Metadatas {}

class MockLists extends Mock implements Lists {}

class MockFollows extends Mock implements Follows {}

void registerNdkFallbackValues() {
  registerFallbackValue(Filter());
  registerFallbackValue(<String>[]);
  registerFallbackValue(Duration.zero);
  registerFallbackValue(
    Nip01Event(pubKey: 'fallback', kind: 1, tags: const [], content: ''),
  );
}
