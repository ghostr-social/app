import 'package:flutter/material.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class SignInScreen extends StatefulWidget {
  const SignInScreen({
    required this.onSubmit,
    this.errorMessage,
    this.isSigningIn = false,
    this.fieldKey,
    this.onBack,
    super.key,
  });

  final ValueChanged<String> onSubmit;
  final String? errorMessage;
  final bool isSigningIn;
  final Key? fieldKey;
  final VoidCallback? onBack;

  @override
  State<SignInScreen> createState() => _SignInScreenState();
}

class _SignInScreenState extends State<SignInScreen> {
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: widget.onBack == null
          ? null
          : AppBar(
              leading: IconButton(
                tooltip: 'Back',
                onPressed: widget.isSigningIn ? null : widget.onBack,
                icon: const Icon(Icons.arrow_back),
              ),
            ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(AppSpacing.xl),
          child: _content(context),
        ),
      ),
    );
  }

  Widget _content(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Spacer(),
        Text(
          'Import your Nostr key',
          style: Theme.of(context).textTheme.headlineMedium,
        ),
        const SizedBox(height: AppSpacing.sm),
        _description(context),
        const SizedBox(height: AppSpacing.xl),
        _secretField(),
        const SizedBox(height: AppSpacing.md),
        _continueButton(),
        const Spacer(),
      ],
    );
  }

  Widget _description(BuildContext context) {
    return Text(
      'Ghostr now starts from a real identity. Paste an `nsec1` key to unlock your profile, feed, and publish flow.',
      style: Theme.of(
        context,
      ).textTheme.bodyLarge?.copyWith(color: AppPalette.mutedForeground),
    );
  }

  Widget _secretField() {
    return TextField(
      key: widget.fieldKey,
      controller: _controller,
      autocorrect: false,
      obscureText: true,
      enabled: !widget.isSigningIn,
      enableSuggestions: false,
      keyboardType: TextInputType.visiblePassword,
      decoration: InputDecoration(
        labelText: 'Nostr secret key',
        errorText: widget.errorMessage,
      ),
      onSubmitted: widget.isSigningIn ? null : widget.onSubmit,
    );
  }

  Widget _continueButton() {
    return ElevatedButton(
      onPressed: widget.isSigningIn
          ? null
          : () => widget.onSubmit(_controller.text),
      child: Text(widget.isSigningIn ? 'Signing in…' : 'Continue'),
    );
  }
}
