import 'package:flutter/material.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

final class PrivateKeyBackupScreen extends StatefulWidget {
  const PrivateKeyBackupScreen({
    required this.secret,
    required this.onFinish,
    this.onCopy,
    this.onSkipPicture,
    this.isFinishing = false,
    this.errorMessage,
    super.key,
  });

  final AuthSecret secret;
  final VoidCallback onFinish;
  final Future<void> Function()? onCopy;
  final VoidCallback? onSkipPicture;
  final bool isFinishing;
  final String? errorMessage;

  @override
  State<PrivateKeyBackupScreen> createState() => _BackupState();
}

final class _BackupState extends State<PrivateKeyBackupScreen> {
  late final _secret = TextEditingController(text: widget.secret.value);
  bool _revealed = false;
  bool _confirmed = false;
  bool _copying = false;
  String? _copyMessage;

  @override
  void dispose() {
    _secret.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Back up your private key')),
      body: ListView(
        padding: const EdgeInsets.all(AppSpacing.xl),
        children: [
          const Text(
            'There is no password reset. If you lose this key and this device, you can lose access to your account. Anyone with it controls your account.',
          ),
          const SizedBox(height: AppSpacing.sm),
          const Text(
            'Your public key is safe to share. Keep this private nsec secret, and store a separate copy: secure storage on this device is not a backup.',
          ),
          const SizedBox(height: AppSpacing.xl),
          _secretField(),
          if (widget.onCopy != null)
            TextButton.icon(
              onPressed: _copying ? null : _copy,
              icon: const Icon(Icons.copy),
              label: Text(
                _copying ? 'Copying private key…' : 'Copy private key',
              ),
            ),
          CheckboxListTile(
            key: const Key('backup-confirmation'),
            value: _confirmed,
            onChanged: widget.isFinishing || _copying
                ? null
                : (value) => setState(() => _confirmed = value ?? false),
            title: const Text('I saved my private key'),
            controlAffinity: ListTileControlAffinity.leading,
          ),
          if (_message != null)
            Text(
              _message!,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
          if (widget.onSkipPicture != null)
            TextButton.icon(
              key: const Key('backup-skip-picture'),
              onPressed: widget.isFinishing || _copying
                  ? null
                  : widget.onSkipPicture,
              icon: const Icon(Icons.image_not_supported_outlined),
              label: const Text('Continue without selected picture'),
            ),
          const SizedBox(height: AppSpacing.md),
          ElevatedButton(
            key: const Key('backup-finish'),
            onPressed: _confirmed && !widget.isFinishing && !_copying
                ? widget.onFinish
                : null,
            child: Text(widget.isFinishing ? 'Creating account…' : 'Finish'),
          ),
        ],
      ),
    );
  }

  Widget _secretField() {
    return TextField(
      key: const Key('backup-private-key-field'),
      controller: _secret,
      readOnly: true,
      obscureText: !_revealed,
      decoration: InputDecoration(
        labelText: 'Nostr private key',
        suffixIcon: IconButton(
          tooltip: _revealed ? 'Hide private key' : 'Reveal private key',
          onPressed: () => setState(() => _revealed = !_revealed),
          icon: Icon(_revealed ? Icons.visibility_off : Icons.visibility),
        ),
      ),
    );
  }

  String? get _message => widget.errorMessage ?? _copyMessage;

  Future<void> _copy() async {
    final copy = widget.onCopy;
    if (copy == null || _copying) return;
    setState(() {
      _copying = true;
      _copyMessage = null;
    });
    try {
      await copy();
      _finishCopy();
    } on AppFailure catch (failure) {
      _finishCopy(failure.message);
    } on Object catch (error, stackTrace) {
      final failure = translatedBoundaryFailure(
        source: 'PrivateKeyBackupScreen.copy',
        message: 'Could not copy the private key.',
        error: error,
        stackTrace: stackTrace,
      );
      _finishCopy(failure.message);
    }
  }

  void _finishCopy([String? message]) {
    if (!mounted) return;
    setState(() {
      _copying = false;
      _copyMessage = message;
    });
  }
}
