import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/app_update_scope.dart';
import 'package:ghostr/core/media/media_picker_port.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/secret_backup_port.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';
import 'package:ghostr/app/session_gate.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_theme.dart';

class GhostrApp extends StatelessWidget {
  const GhostrApp({required this.dependencies, super.key});

  final AppDependencies dependencies;

  @override
  Widget build(BuildContext context) {
    final application = MultiRepositoryProvider(
      providers: _repositoryProviders(),
      child: _sessionScope(AppControllerFactory(dependencies)),
    );
    final runtime = dependencies.appUpdateRuntime;
    if (runtime == null) return application;
    return AppUpdateScope(
      create: () => AppUpdateCubit(runtime.dependencies),
      disposeRuntime: runtime.dispose,
      child: application,
    );
  }

  List<RepositoryProvider<Object>> _repositoryProviders() {
    return [
      RepositoryProvider<SessionRepository>.value(
        value: dependencies.sessionRepository,
      ),
      RepositoryProvider<SecretBackupPort>.value(
        value: dependencies.secretBackupPort,
      ),
      RepositoryProvider<AppSettingsRepository>.value(
        value: dependencies.appSettingsRepository,
      ),
      RepositoryProvider<ActivityRepository>.value(
        value: dependencies.activityRepository,
      ),
      RepositoryProvider<MediaPickerPort>.value(
        value: dependencies.mediaPickerPort,
      ),
      RepositoryProvider<VideoPlaybackPort>.value(
        value: dependencies.videoPlaybackPort,
      ),
    ];
  }

  Widget _sessionScope(AppControllerFactory controllers) {
    return MultiBlocProvider(
      providers: [
        BlocProvider(
          create: (context) =>
              SessionCubit(context.read<SessionRepository>())..restore(),
        ),
        BlocProvider(
          create: (_) => AccountCreationCubit(
            dependencies.accountGenerator,
            dependencies.accountProvisioningRepository,
            dependencies.profileMetadataRepository,
            dependencies.profileImageWorkflow,
          )..restorePending(),
        ),
      ],
      child: _materialApp(controllers),
    );
  }

  Widget _materialApp(AppControllerFactory controllers) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'Ghostr',
      theme: buildAppTheme(),
      home: SessionGate(controllers: controllers),
    );
  }
}
