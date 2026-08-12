import 'package:flutter/material.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class SettingsBlockedAccountsSection extends StatelessWidget {
  const SettingsBlockedAccountsSection({
    required this.onOpenBlockedAccounts,
    super.key,
  });

  final VoidCallback? onOpenBlockedAccounts;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text('Blocked accounts', style: Theme.of(context).textTheme.titleLarge),
        const SizedBox(height: AppSpacing.xs),
        const Text('Creators you block never appear in your feeds.'),
        const SizedBox(height: AppSpacing.sm),
        ListTile(
          key: const Key('blocked-accounts-entry'),
          title: const Text('Manage blocked accounts'),
          trailing: const Icon(Icons.chevron_right),
          onTap: onOpenBlockedAccounts,
        ),
      ],
    );
  }
}
