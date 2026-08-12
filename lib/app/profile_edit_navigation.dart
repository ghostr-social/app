import 'package:flutter/material.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

Future<void> openProfileEditor(
  BuildContext context,
  UserSession session,
  AppControllerFactory controllers,
  ValueChanged<ProfileSummary> onSaved,
) async {
  final updated = await Navigator.of(
    context,
  ).push(AppRouter.editProfile(session: session, controllers: controllers));
  if (context.mounted && updated != null) onSaved(updated);
}
