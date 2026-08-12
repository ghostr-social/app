import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/social/domain/blocked_account.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_cubit.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

class BlockedAccountsScreen extends StatelessWidget {
  const BlockedAccountsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Blocked accounts')),
      body: BlocBuilder<BlockedAccountsCubit, BlockedAccountsState>(
        builder: _buildAccounts,
      ),
    );
  }

  Widget _buildAccounts(BuildContext context, BlockedAccountsState state) {
    return switch (state) {
      BlockedAccountsLoading() => const LoadingPanel(
          label: 'Loading blocked accounts',
        ),
      BlockedAccountsEmpty() => _emptyAccounts(),
      BlockedAccountsLoaded(accounts: final accounts) => _accountList(
          context,
          accounts,
        ),
      BlockedAccountsFailure(message: final message) => _errorAccounts(
          context,
          message,
        ),
    };
  }

  Widget _emptyAccounts() {
    return const AsyncStatePanel(
      icon: Icons.block,
      title: 'No blocked accounts',
      message: 'Creators you block from the feed or their profile are '
          'listed here so you can revert it.',
    );
  }

  Widget _errorAccounts(BuildContext context, String message) {
    return AsyncStatePanel(
      icon: Icons.error_outline,
      title: 'Blocked accounts unavailable',
      message: message,
      actionLabel: 'Retry',
      onAction: context.read<BlockedAccountsCubit>().load,
    );
  }

  Widget _accountList(BuildContext context, List<BlockedAccount> accounts) {
    return ListView.separated(
      padding: const EdgeInsets.all(AppSpacing.lg),
      itemCount: accounts.length,
      separatorBuilder: (_, __) => const SizedBox(height: AppSpacing.sm),
      itemBuilder: (_, index) => _accountTile(context, accounts[index]),
    );
  }

  Widget _accountTile(BuildContext context, BlockedAccount account) {
    return ListTile(
      tileColor: Theme.of(context).colorScheme.surface,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppRadius.control),
      ),
      title: Text(account.label, maxLines: 1, overflow: TextOverflow.ellipsis),
      subtitle: account.displayName == null ? null : Text(account.shortId),
      trailing: TextButton(
        onPressed: () =>
            context.read<BlockedAccountsCubit>().unblock(account.id),
        child: const Text('Unblock'),
      ),
    );
  }
}
