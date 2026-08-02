import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/search_screen.dart';

Widget searchScreenHarness(
  VideoSearchRepository repository, {
  ValueChanged<String>? onOpenProfile,
}) {
  return MaterialApp(
    home: Scaffold(
      body: BlocProvider(
        create: (_) => SearchCubit(repository),
        child: SearchScreen(onOpenProfile: onOpenProfile ?? (_) {}),
      ),
    ),
  );
}
