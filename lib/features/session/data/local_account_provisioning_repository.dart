import 'dart:convert';

import 'package:ghostr/core/storage/preference_storage_guard.dart';
import 'package:ghostr/core/storage/secret_store.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/session/domain/account_provisioning_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/generated_nostr_account.dart';
import 'package:ghostr/features/session/domain/nostr_identity_deriver.dart';
import 'package:ghostr/features/session/domain/nostr_session_port.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:shared_preferences/shared_preferences.dart';

final class LocalAccountProvisioningRepository
    implements AccountProvisioningRepository {
  const LocalAccountProvisioningRepository(
    this._preferences,
    this._secretStores,
    this._identityDeriver,
    this._nostrSession,
  );

  static const _draftKey = 'ghostr.account.provisioning.v1';
  final SharedPreferences _preferences;
  final AccountProvisioningSecretStores _secretStores;
  final NostrIdentityDeriver _identityDeriver;
  final NostrSessionPort _nostrSession;

  @override
  Future<void> stage(PendingAccountSetup setup) async {
    await requirePreferenceWrite(
      'Could not save account setup.',
      () => _preferences.setString(_draftKey, jsonEncode(_encode(setup))),
    );
    try {
      await _secretStores.pending.write(setup.account.secret.value);
    } on Object {
      await _removeDraft();
      rethrow;
    }
  }

  @override
  Future<RestoredPendingAccount?> restorePending() async {
    return guardPreferenceStorage('Could not read account setup.', () async {
      final raw = _preferences.getString(_draftKey);
      final encodedSecret = await _secretStores.pending.read();
      if (raw == null && encodedSecret == null) return null;
      if (encodedSecret == null) return _discardOrphan();
      final GeneratedNostrAccount account;
      try {
        account = _account(AuthSecret.parse(encodedSecret));
      } on FormatException {
        return _discardOrphan();
      }
      if (raw == null) return PendingAccountProfileRecovery(account);
      try {
        return _decode(raw, account);
      } on FormatException {
        return PendingAccountProfileRecovery(account);
      }
    });
  }

  @override
  Future<UserSession> activate(PendingAccountSetup setup) async {
    _validateIdentity(setup);
    await _nostrSession.activate(setup.account.secret, setup.account.identity);
    return UserSession.fromIdentity(setup.account.identity);
  }

  @override
  Future<void> commit(PendingAccountSetup setup) async {
    _validateIdentity(setup);
    await _secretStores.active.write(setup.account.secret.value);
    await discard();
  }

  @override
  Future<void> discard() async {
    await _removeDraft();
    await _secretStores.pending.clear();
  }

  Map<String, Object?> _encode(PendingAccountSetup setup) {
    return {
      'npub': setup.account.identity.npub.value,
      'displayName': setup.metadata.displayName.value,
      'handle': setup.metadata.handle.value,
      'pictureUrl': setup.metadata.pictureUrl?.value,
    };
  }

  GeneratedNostrAccount _account(AuthSecret secret) {
    return GeneratedNostrAccount(
      secret: secret,
      identity: _identityDeriver.derive(secret),
    );
  }

  PendingAccountSetup _decode(String raw, GeneratedNostrAccount account) {
    final payload = jsonDecode(raw);
    if (payload is! Map<String, dynamic>) throw const FormatException();
    if (_required(payload, 'npub') != account.identity.npub.value) {
      throw const FormatException('Pending account does not match its key.');
    }
    return PendingAccountSetup(account: account, metadata: _metadata(payload));
  }

  ProfileMetadata _metadata(Map<String, dynamic> payload) {
    return ProfileMetadata.parse(
      displayName: _required(payload, 'displayName'),
      handle: _required(payload, 'handle'),
      pictureUrl: _optional(payload, 'pictureUrl'),
    );
  }

  String _required(Map<String, dynamic> payload, String key) {
    return _optional(payload, key) ?? (throw const FormatException());
  }

  String? _optional(Map<String, dynamic> payload, String key) {
    final value = payload[key];
    if (value == null || value is String) return value as String?;
    throw const FormatException();
  }

  void _validateIdentity(PendingAccountSetup setup) {
    final derived = _identityDeriver.derive(setup.account.secret);
    if (derived.npub != setup.account.identity.npub) {
      throw StateError('Generated account identity does not match its key.');
    }
  }

  Future<RestoredPendingAccount?> _discardOrphan() async {
    await discard();
    return null;
  }

  Future<void> _removeDraft() {
    if (!_preferences.containsKey(_draftKey)) return Future.value();
    return requirePreferenceWrite(
      'Could not clear account setup.',
      () => _preferences.remove(_draftKey),
    );
  }
}

final class AccountProvisioningSecretStores {
  const AccountProvisioningSecretStores({
    required this.pending,
    required this.active,
  });

  final SecretStore pending;
  final SecretStore active;
}
