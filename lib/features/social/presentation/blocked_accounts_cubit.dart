import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/social/domain/blocked_account.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

sealed class BlockedAccountsState {
  const BlockedAccountsState();
}

class BlockedAccountsLoading extends BlockedAccountsState {
  const BlockedAccountsLoading();
}

class BlockedAccountsEmpty extends BlockedAccountsState {
  const BlockedAccountsEmpty();
}

class BlockedAccountsLoaded extends BlockedAccountsState {
  factory BlockedAccountsLoaded(List<BlockedAccount> accounts) {
    if (accounts.isEmpty) {
      throw StateError('Loaded blocked accounts cannot be empty.');
    }
    return BlockedAccountsLoaded._(
      List<BlockedAccount>.unmodifiable(accounts),
    );
  }

  const BlockedAccountsLoaded._(this.accounts);

  final List<BlockedAccount> accounts;
}

class BlockedAccountsFailure extends BlockedAccountsState {
  const BlockedAccountsFailure(this.message);

  final String message;
}

class BlockedAccountsCubit extends DisposalSafeCubit<BlockedAccountsState> {
  BlockedAccountsCubit(this._social, this._metadata)
      : super(const BlockedAccountsLoading());

  final SocialGraphRepository _social;
  final ProfileMetadataRepository _metadata;
  var _request = 0;

  Future<void> load() {
    return _run('load', _listState);
  }

  /// Reverts one block, then reflects what the block list holds now.
  Future<void> unblock(ProfileId profileId) {
    return _run('unblock', () async {
      await _social.toggleBlock(profileId);
      return _listState();
    });
  }

  Future<BlockedAccountsState> _listState() async {
    final accounts = await _describeBlocked();
    return accounts.isEmpty
        ? const BlockedAccountsEmpty()
        : BlockedAccountsLoaded(accounts);
  }

  Future<void> _run(
    String action,
    Future<BlockedAccountsState> Function() transition,
  ) async {
    final request = ++_request;
    emit(const BlockedAccountsLoading());
    try {
      _emitIfCurrent(request, await transition());
    } on AppFailure catch (failure) {
      _emitIfCurrent(request, BlockedAccountsFailure(failure.message));
    } on Object catch (error, stackTrace) {
      _emitIfCurrent(
        request,
        BlockedAccountsFailure(_unexpected(action, error, stackTrace)),
      );
    }
  }

  Future<List<BlockedAccount>> _describeBlocked() async {
    final blocked = await _social.loadBlockedProfiles();
    final accounts = <BlockedAccount>[
      for (final id in blocked)
        BlockedAccount(id: id, displayName: await _cachedName(id)),
    ];
    accounts.sort(
      (left, right) =>
          left.label.toLowerCase().compareTo(right.label.toLowerCase()),
    );
    return accounts;
  }

  // A missing or unreadable cached profile only costs the display name.
  Future<String?> _cachedName(ProfileId id) async {
    try {
      return (await _metadata.loadCached(id))?.displayName;
    } on Object {
      return null;
    }
  }

  void _emitIfCurrent(int request, BlockedAccountsState next) {
    if (!isClosed && request == _request) emit(next);
  }

  String _unexpected(String action, Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'BlockedAccountsCubit.$action',
      message: 'Could not update blocked accounts. Try again.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }
}
