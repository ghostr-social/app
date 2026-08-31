import 'package:ndk/ndk.dart';

import 'warp_feed_nostr_account.dart';

const _offlineAccountPrivateKey =
    '1111111111111111111111111111111111111111111111111111111111111111';

WarpFeedNostrAccount warpOfflineRestartAccount() {
  return WarpFeedNostrAccount.fromPrivateKey(_offlineAccountPrivateKey);
}

String warpOfflineRestartPublicKey() {
  return const Bip340EventSignerFactory().derivePublicKey(
    _offlineAccountPrivateKey,
  );
}
