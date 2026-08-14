import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class AppUpdateOfferOverlay extends StatelessWidget {
  const AppUpdateOfferOverlay({required this.child, this.cubit, super.key});

  final Widget child;
  final AppUpdateCubit? cubit;

  @override
  Widget build(BuildContext context) {
    final updater = cubit ?? context.read<AppUpdateCubit>();
    return BlocBuilder<AppUpdateCubit, AppUpdateState>(
      bloc: updater,
      builder: (context, state) => Stack(
        children: [
          child,
          if (state case final AppUpdateOfferedState offered)
            _offer(updater, offered),
        ],
      ),
    );
  }

  Widget _offer(AppUpdateCubit updater, AppUpdateOfferedState offered) {
    final version = offered.release.versionName;
    return Positioned(
      bottom: kBottomNavigationBarHeight + AppSpacing.sm,
      left: AppSpacing.sm,
      right: AppSpacing.sm,
      child: SafeArea(
        top: false,
        left: false,
        right: false,
        child: Align(
          alignment: Alignment.topCenter,
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 560),
            child: Semantics(
              container: true,
              liveRegion: true,
              label: 'Ghostr $version update available',
              child: _OfferCard(updater: updater, offered: offered),
            ),
          ),
        ),
      ),
    );
  }
}

class _OfferCard extends StatelessWidget {
  const _OfferCard({required this.updater, required this.offered});

  final AppUpdateCubit updater;
  final AppUpdateOfferedState offered;

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 6,
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.md),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Ghostr ${offered.release.versionName} is available',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: AppSpacing.xs),
            _message(context),
            const SizedBox(height: AppSpacing.sm),
            if (offered.pendingAction case final action?) ...[
              Semantics(
                container: true,
                label: _pendingLabel(action),
                child: const ExcludeSemantics(child: LinearProgressIndicator()),
              ),
              const SizedBox(height: AppSpacing.sm),
            ],
            _actions(),
          ],
        ),
      ),
    );
  }

  Widget _message(BuildContext context) {
    final error = offered.message;
    if (error == null) {
      return const Text('Update when you are ready. Your video keeps playing.');
    }
    return Semantics(
      container: true,
      liveRegion: true,
      label: error,
      child: ExcludeSemantics(
        child: Text(
          error,
          style: TextStyle(color: Theme.of(context).colorScheme.error),
        ),
      ),
    );
  }

  Widget _actions() {
    final enabled = offered.pendingAction == null;
    return OverflowBar(
      alignment: MainAxisAlignment.end,
      spacing: AppSpacing.xs,
      overflowSpacing: AppSpacing.xs,
      children: [
        TextButton(
          onPressed: enabled ? _decline : null,
          child: const Text('Skip this version'),
        ),
        FilledButton(
          onPressed: enabled ? _accept : null,
          child: const Text('Update'),
        ),
      ],
    );
  }

  String _pendingLabel(AppUpdateOfferAction action) => switch (action) {
    AppUpdateOfferAction.accepting => 'Starting update',
    AppUpdateOfferAction.declining => 'Saving skipped version',
  };

  void _accept() {
    unawaited(updater.acceptOffer(offered.release.versionCode));
  }

  void _decline() {
    unawaited(updater.declineOffer(offered.release.versionCode));
  }
}
