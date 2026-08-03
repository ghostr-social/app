import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/metadata_creator_search_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_profile_search_port.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('search metadata becomes creator profiles with npub identities',
      () async {
    final source = MetadataCreatorSearchSource(_StubPort([
      Metadata()
        ..pubKey = testViewerPublicKey
        ..name = 'Alice'
        ..picture = 'https://example.com/alice.png',
      Metadata()..pubKey = testCreatorPublicKey,
      Metadata()..pubKey = 'not-a-key',
    ]));

    final creators = await source.searchCreators('ali');

    expect(creators, hasLength(2));
    expect(creators.first.id, testViewerNpub);
    expect(creators.first.displayName, 'Alice');
    expect(creators.first.handle, '@$testViewerNpub');
    expect(creators.first.avatarUrl, 'https://example.com/alice.png');
    // A nameless profile falls back to a shortened npub, never raw hex.
    expect(creators.last.displayName, endsWith('…'));
    expect(creators.last.displayName, startsWith('npub1'));
  });
}

class _StubPort implements NostrProfileSearchPort {
  _StubPort(this.metadata);

  final List<Metadata> metadata;

  @override
  Future<List<Metadata>> searchProfiles(String query) async => metadata;
}
