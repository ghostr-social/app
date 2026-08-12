import 'package:flutter/material.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

final class OnboardingWelcomeScreen extends StatelessWidget {
  const OnboardingWelcomeScreen({
    required this.onCreateAccount,
    required this.onUseExistingKey,
    super.key,
  });

  final VoidCallback onCreateAccount;
  final VoidCallback onUseExistingKey;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(AppSpacing.xl),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Spacer(),
              Text(
                'Welcome to Ghostr',
                style: Theme.of(context).textTheme.headlineMedium,
              ),
              const SizedBox(height: AppSpacing.sm),
              const Text(
                'Your Nostr account is your identity across the open network.',
              ),
              const SizedBox(height: AppSpacing.xl),
              ElevatedButton(
                onPressed: onCreateAccount,
                child: const Text('Create a Nostr account'),
              ),
              const SizedBox(height: AppSpacing.sm),
              OutlinedButton(
                onPressed: onUseExistingKey,
                child: const Text('Use an existing key'),
              ),
              const Spacer(),
            ],
          ),
        ),
      ),
    );
  }
}
