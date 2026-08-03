import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/core/media/media_picker_port.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';
import 'package:ghostr/features/compose/presentation/compose_screen.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';

import 'fake_media_ports.dart';
import 'compose_dependencies.dart';
import 'sample_data.dart';

Widget composeScreenHarness({
  required VideoPublishingRepository publishing,
  required ActivityRepository activity,
  required MediaPickerPort picker,
}) {
  final cubit = ComposeCubit(buildComposeDependencies(
    publishing: publishing,
    activity: activity,
    picker: picker,
  ));
  return MaterialApp(
    home: BlocProvider.value(
      value: cubit..recoverLostVideo(),
      child: Scaffold(
        body: ComposeScreen(
          session: sampleSession(),
          playbackPort: FakeVideoPlaybackPort(),
          isActive: true,
        ),
      ),
    ),
  );
}
