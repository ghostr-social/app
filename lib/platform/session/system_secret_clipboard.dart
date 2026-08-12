import 'package:flutter/services.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/secret_backup_port.dart';

typedef ClipboardWriter = Future<void> Function(ClipboardData data);

final class SystemSecretClipboard implements SecretBackupPort {
  SystemSecretClipboard({ClipboardWriter? writer})
    : _writer = writer ?? Clipboard.setData;

  final ClipboardWriter _writer;

  @override
  Future<void> copy(AuthSecret secret) async {
    try {
      await _writer(ClipboardData(text: secret.value));
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.session.secret_clipboard',
        message: 'Could not copy the private key.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
