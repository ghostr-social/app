import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ndk/ndk.dart';

/// Builds the domain creator profile shown across feeds and search results
/// from a Nostr public key and its (possibly missing) kind-0 metadata.
ProfileSummary creatorProfileSummary(String publicKeyHex, Metadata? metadata) {
  final npub = Nip19.encodePubKey(publicKeyHex);
  final name = metadata?.getName();
  final hasName =
      name != null && name != publicKeyHex && name.trim().isNotEmpty;
  return ProfileSummary(
    id: ProfileId.parse(npub),
    displayName: hasName ? name : '${npub.substring(0, 12)}…',
    handle: '@$npub',
    avatarUrl: metadata?.picture,
  );
}
