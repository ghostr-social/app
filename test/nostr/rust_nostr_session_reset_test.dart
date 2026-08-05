import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/session/domain/nostr_session_port.dart';
import 'package:ghostr/platform/nostr/rust_nostr_session.dart';

const _nsec = 'nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5';
final _identity = NostrIdentity.parse(
  publicKeyHex:
      '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e',
  npub: 'npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg',
);

void main() {
  test('resets Rust before activating the local signer', () async {
    final calls = <String>[];
    final session = RustNostrSession(
      local: _RecordingSession(calls),
      reset: (account) async => calls.add('reset:${account?.value}'),
    );

    await session.activate(AuthSecret.parse(_nsec), _identity);

    expect(calls, ['reset:${_identity.publicKeyHex.value}', 'activate']);
  });

  test('resets Rust before deactivating the local signer', () async {
    final calls = <String>[];
    final session = RustNostrSession(
      local: _RecordingSession(calls),
      reset: (account) async => calls.add('reset:${account?.value}'),
    );

    await session.deactivate();

    expect(calls, ['reset:null', 'deactivate']);
  });

  test('leaves the local signer untouched when reset fails', () async {
    final calls = <String>[];
    final session = RustNostrSession(
      local: _RecordingSession(calls),
      reset: (_) async => throw StateError('engine unavailable'),
    );

    await expectLater(
      session.deactivate(),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Could not reset the Nostr engine session.',
        ),
      ),
    );

    expect(calls, isEmpty);
  });
}

class _RecordingSession implements NostrSessionPort {
  _RecordingSession(this.calls);

  final List<String> calls;

  @override
  Future<void> activate(AuthSecret secret, NostrIdentity identity) async {
    calls.add('activate');
  }

  @override
  Future<void> deactivate() async {
    calls.add('deactivate');
  }
}
