import 'package:ghostr/core/nostr/nostr_event_identity.dart';

typedef ActiveAccountReader = NostrPublicKeyHex Function();

class AccountStorageScope {
  const AccountStorageScope(this._activeAccount);

  final ActiveAccountReader _activeAccount;

  AccountStorageKey capture() => AccountStorageKey(_activeAccount());
}

class AccountStorageKey {
  const AccountStorageKey(this.account);

  final NostrPublicKeyHex account;

  String key(String namespace) => '$namespace.$account';

  @override
  bool operator ==(Object other) {
    return other is AccountStorageKey && other.account == account;
  }

  @override
  int get hashCode => account.hashCode;
}
