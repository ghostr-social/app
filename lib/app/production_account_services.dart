import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:ghostr/app/production_nostr_services.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_repository.dart';
import 'package:ghostr/features/profile/domain/profile_image_workflow.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/session/data/local_account_provisioning_repository.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/data/pending_first_session_repository.dart';
import 'package:ghostr/features/session/data/profiled_session_repository.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';
import 'package:ghostr/features/session/domain/account_provisioning_repository.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/platform/media/image_picker_profile_image_picker.dart';
import 'package:ghostr/platform/storage/secure_secret_store.dart';
import 'package:image_picker/image_picker.dart';
import 'package:shared_preferences/shared_preferences.dart';

final class ProductionAccountServicesInputs {
  const ProductionAccountServicesInputs({
    required this.preferences,
    required this.nostr,
  });

  final SharedPreferences preferences;
  final ProductionNostrServices nostr;
}

final class ProductionAccountServices {
  const ProductionAccountServices._(this._scoped, this._session, this._profile);

  final _ProductionScopedAccountServices _scoped;
  final _ProductionSessionServices _session;
  final _ProductionProfileServices _profile;

  AccountStorageScope get accountScope => _scoped.accountScope;
  LocalWatchHistoryRepository get watchHistory => _scoped.watchHistory;
  SessionRepository get sessionRepository => _session.repository;
  AccountProvisioningRepository get provisioningRepository =>
      _session.provisioningRepository;
  ProfileMetadataRepository get profileMetadataRepository =>
      _profile.metadataRepository;
  ProfileImageWorkflow get profileImageWorkflow => _profile.imageWorkflow;
}

ProductionAccountServices buildProductionAccountServices(
  ProductionAccountServicesInputs inputs,
) {
  final scoped = _buildScopedAccountServices(inputs);
  final profile = _buildProfileServices(inputs);
  final session = _buildSessionServices(inputs, profile.metadataRepository);
  return ProductionAccountServices._(scoped, session, profile);
}

_ProductionScopedAccountServices _buildScopedAccountServices(
  ProductionAccountServicesInputs inputs,
) {
  final accountScope = AccountStorageScope(
    () => inputs.nostr.eventClient.publicKeyHex,
  );
  return _ProductionScopedAccountServices(
    accountScope,
    LocalWatchHistoryRepository(inputs.preferences, accountScope: accountScope),
  );
}

_ProductionProfileServices _buildProfileServices(
  ProductionAccountServicesInputs inputs,
) {
  final metadataRepository = NostrProfileMetadataRepository(
    client: inputs.nostr.eventClient,
    cache: LocalProfileMetadataCache(inputs.preferences),
  );
  return _ProductionProfileServices(
    metadataRepository,
    _buildProfileImageWorkflow(inputs.nostr),
  );
}

_ProductionSessionServices _buildSessionServices(
  ProductionAccountServicesInputs inputs,
  ProfileMetadataRepository profileMetadataRepository,
) {
  const secureStorage = FlutterSecureStorage();
  final activeSecrets = SecureSecretStore(secureStorage);
  final provisioning = LocalAccountProvisioningRepository(
    inputs.preferences,
    AccountProvisioningSecretStores(
      pending: SecureSecretStore(
        secureStorage,
        storageKey: 'ghostr.viewer.pendingSecret',
      ),
      active: activeSecrets,
    ),
    const NdkNostrIdentityDeriver(),
    inputs.nostr.adapters.session,
  );
  final sessions = ProfiledSessionRepository(
    SecureSessionRepository(
      activeSecrets,
      const NdkNostrIdentityDeriver(),
      inputs.nostr.adapters.session,
    ),
    profileMetadataRepository,
  );
  return _ProductionSessionServices(
    PendingFirstSessionRepository(sessions, provisioning),
    provisioning,
  );
}

ProfileImageWorkflow _buildProfileImageWorkflow(ProductionNostrServices nostr) {
  final uploader = nostr.profileImageUploader;
  if (uploader == null) return const ProfileImageWorkflow.disabled();
  return ProfileImageWorkflow(
    ImagePickerProfileImagePicker(ImagePicker()),
    uploader,
  );
}

final class _ProductionScopedAccountServices {
  const _ProductionScopedAccountServices(this.accountScope, this.watchHistory);

  final AccountStorageScope accountScope;
  final LocalWatchHistoryRepository watchHistory;
}

final class _ProductionSessionServices {
  const _ProductionSessionServices(
    this.repository,
    this.provisioningRepository,
  );

  final SessionRepository repository;
  final AccountProvisioningRepository provisioningRepository;
}

final class _ProductionProfileServices {
  const _ProductionProfileServices(this.metadataRepository, this.imageWorkflow);

  final ProfileMetadataRepository metadataRepository;
  final ProfileImageWorkflow imageWorkflow;
}
