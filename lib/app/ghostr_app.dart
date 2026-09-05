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
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_theme.dart';

class GhostrApp extends StatefulWidget {
  const GhostrApp({required this.dependencies, this.feedFocus, super.key});

  final AppDependencies dependencies;
  final FeedFocusSink? feedFocus;

  @override
  State<GhostrApp> createState() => _GhostrAppState();
}

class _GhostrAppState extends State<GhostrApp> {
  late AppControllerFactory _controllers;

  @override
  void initState() {
    super.initState();
    _controllers = _createControllers();
  }

  @override
  void didUpdateWidget(covariant GhostrApp oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (identical(oldWidget.dependencies, widget.dependencies) &&
        identical(oldWidget.feedFocus, widget.feedFocus)) {
      return;
    }
    _controllers = _createControllers();
  }

  @override
  Widget build(BuildContext context) {
    final application = MultiRepositoryProvider(
      providers: _repositoryProviders(),
      child: _sessionScope(_controllers),
    );
    final runtime = widget.dependencies.appUpdateRuntime;
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
        value: widget.dependencies.sessionRepository,
      ),
      RepositoryProvider<SecretBackupPort>.value(
        value: widget.dependencies.secretBackupPort,
      ),
      RepositoryProvider<AppSettingsRepository>.value(
        value: widget.dependencies.appSettingsRepository,
      ),
      RepositoryProvider<ActivityRepository>.value(
        value: widget.dependencies.activityRepository,
      ),
      RepositoryProvider<MediaPickerPort>.value(
        value: widget.dependencies.mediaPickerPort,
      ),
      RepositoryProvider<VideoPlaybackPort>.value(
        value: widget.dependencies.videoPlaybackPort,
      ),
    ];
  }

  Widget _sessionScope(AppControllerFactory controllers) {
    return MultiBlocProvider(
      key: ValueKey(controllers),
      providers: [
        BlocProvider(
          create: (context) =>
              SessionCubit(context.read<SessionRepository>())..restore(),
        ),
        BlocProvider(
          create: (_) => AccountCreationCubit(
            widget.dependencies.accountGenerator,
            widget.dependencies.accountProvisioningRepository,
            widget.dependencies.profileMetadataRepository,
            widget.dependencies.profileImageWorkflow,
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

  AppControllerFactory _createControllers() {
    return AppControllerFactory(
      widget.dependencies,
      feedFocus: widget.feedFocus,
    );
  }
}
