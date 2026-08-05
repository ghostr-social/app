import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/trending_hashtags.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/search_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/trending_hashtags_cubit.dart';

Widget searchScreenHarness(
  VideoSearchRepository repository, {
  TrendingHashtagsSource? trending,
  ValueChanged<ProfileId>? onOpenProfile,
  ValueChanged<String>? onOpenFeed,
  VideoSearchUpdates? updates,
}) {
  return MaterialApp(
    home: Scaffold(
      body: MultiBlocProvider(
        providers: [
          BlocProvider<SearchCubit>(
            create: (_) => SearchCubit(
              repository,
              updates: updates,
            ),
          ),
          BlocProvider<TrendingHashtagsCubit>(
            create: (_) =>
                TrendingHashtagsCubit(trending ?? const _NoTrending())..load(),
          ),
        ],
        child: SearchScreen(
          onOpenProfile: onOpenProfile ?? (_) {},
          onOpenFeed: onOpenFeed ?? (_) {},
        ),
      ),
    ),
  );
}

class _NoTrending implements TrendingHashtagsSource {
  const _NoTrending();

  @override
  Future<List<String>> trendingHashtags() async => const <String>[];
}
