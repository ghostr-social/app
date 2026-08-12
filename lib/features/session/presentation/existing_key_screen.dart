import 'package:flutter/material.dart';
import 'package:ghostr/features/session/presentation/sign_in_screen.dart';

final class ExistingKeyScreen extends StatelessWidget {
  const ExistingKeyScreen({
    required this.onSubmit,
    this.errorMessage,
    this.isSigningIn = false,
    this.onBack,
    super.key,
  });

  final ValueChanged<String> onSubmit;
  final String? errorMessage;
  final bool isSigningIn;
  final VoidCallback? onBack;

  @override
  Widget build(BuildContext context) {
    return SignInScreen(
      onSubmit: onSubmit,
      errorMessage: errorMessage,
      isSigningIn: isSigningIn,
      fieldKey: const Key('existing-key-nsec-field'),
      onBack: onBack,
    );
  }
}
