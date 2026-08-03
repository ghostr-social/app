import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/ndk_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_query_port.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ndk/ndk.dart';

class _UnusedQuery implements NostrVideoEventQueryPort {
  int callCount = 0;

  @override
  Future<List<Nip01Event>> loadVideoEvents({
    Set<NostrPublicKeyHex>? authorPublicKeys,
    String? searchQuery,
  }) async {
    callCount += 1;
    return const [];
  }

  @override
  Future<Map<NostrPublicKeyHex, Metadata>> loadMetadataBatch(
    Set<NostrPublicKeyHex> publicKeys,
  ) async =>
      const {};
}

void main() {
  test('does not query relays for a non-Nostr creator identifier', () async {
    final query = _UnusedQuery();
    final source = NdkVideoRemoteSource(query);

    final posts = await source.loadRemoteFeed(
      creatorIds: {ProfileId.parse('not-a-nostr-profile')},
    );

    expect(posts, isEmpty);
    expect(query.callCount, 0);
  });
}
