import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/search_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/trending_hashtags_cubit.dart';

/// The search tab: search plus trending discovery over one screen.
class SearchTab extends StatelessWidget {
  const SearchTab({
    required this.createSearchCubit,
    required this.createTrendingCubit,
    required this.onOpenProfile,
    required this.onOpenFeed,
    super.key,
  });

  final SearchCubit Function() createSearchCubit;
  final TrendingHashtagsCubit Function() createTrendingCubit;
  final ValueChanged<ProfileId> onOpenProfile;
  final ValueChanged<String> onOpenFeed;

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: [
        BlocProvider<SearchCubit>(create: (_) => createSearchCubit()),
        BlocProvider<TrendingHashtagsCubit>(
          create: (_) => createTrendingCubit(),
        ),
      ],
      child: SearchScreen(
        onOpenProfile: onOpenProfile,
        onOpenFeed: onOpenFeed,
      ),
    );
  }
}
