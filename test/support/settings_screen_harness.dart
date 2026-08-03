import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';
import 'package:ghostr/features/settings/presentation/settings_screen.dart';

Widget settingsScreenHarness(
  AppSettingsRepository repository, {
  VoidCallback? onOpenWatchHistory,
}) {
  return MaterialApp(
    home: BlocProvider(
      create: (_) => SettingsCubit(repository)..load(),
      child: SettingsScreen(onOpenWatchHistory: onOpenWatchHistory),
    ),
  );
}
