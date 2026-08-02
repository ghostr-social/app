import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/settings/presentation/settings_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_screen.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

abstract final class AppRouter {
  static Route<void> profile({
    required UserSession session,
    required ProfileId profileId,
    required AppControllerFactory controllers,
    required VoidCallback onSignedOut,
  }) {
    return MaterialPageRoute<void>(
      builder: (_) => BlocProvider(
        create: (_) => controllers.profile(session.profile, profileId)..load(),
        child: ProfileScreen(onSignedOut: onSignedOut),
      ),
    );
  }

  static Route<void> settings(AppControllerFactory controllers) {
    return MaterialPageRoute<void>(
      builder: (_) => BlocProvider(
        create: (_) => controllers.settings()..load(),
        child: const SettingsScreen(),
      ),
    );
  }
}
