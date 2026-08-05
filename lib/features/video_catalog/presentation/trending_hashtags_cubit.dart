import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/trending_hashtags.dart';

sealed class TrendingHashtagsState {
  const TrendingHashtagsState();
}

class TrendingHashtagsLoading extends TrendingHashtagsState {
  const TrendingHashtagsLoading();
}

class TrendingHashtagsReady extends TrendingHashtagsState {
  TrendingHashtagsReady(List<String> tags)
      : tags = List<String>.unmodifiable(tags);

  final List<String> tags;
}

class TrendingHashtagsUnavailable extends TrendingHashtagsState {
  const TrendingHashtagsUnavailable();
}

/// Discovery garnish for the search screen: failures stay silent and the
/// section simply hides when nothing is trending.
class TrendingHashtagsCubit extends DisposalSafeCubit<TrendingHashtagsState> {
  TrendingHashtagsCubit(this._source) : super(const TrendingHashtagsLoading());

  final TrendingHashtagsSource _source;

  Future<void> load() async {
    try {
      final tags = await _source.trendingHashtags();
      emit(tags.isEmpty
          ? const TrendingHashtagsUnavailable()
          : TrendingHashtagsReady(tags));
    } on Object {
      emit(const TrendingHashtagsUnavailable());
    }
  }
}
