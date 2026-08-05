import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/search_state.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class SearchResultsActions {
  const SearchResultsActions({
    required this.onOpenProfile,
    required this.onOpenFeed,
    required this.onLoadMore,
  });

  final ValueChanged<ProfileId> onOpenProfile;
  final VoidCallback onOpenFeed;
  final VoidCallback onLoadMore;
}

/// Creators row plus a paged list of matching videos with a feed entry.
class SearchResultsView extends StatelessWidget {
  const SearchResultsView({
    required this.results,
    required this.actions,
    super.key,
  });

  final SearchLoaded results;
  final SearchResultsActions actions;

  @override
  Widget build(BuildContext context) {
    return NotificationListener<ScrollNotification>(
      onNotification: _maybeLoadMore,
      child: CustomScrollView(
        slivers: [
          if (results.creators.isNotEmpty)
            SliverToBoxAdapter(child: _creatorsSection(context)),
          // The header always renders: the feed entry must stay reachable
          // even when only creators matched.
          SliverToBoxAdapter(child: _videosHeader(context)),
          if (results.videos.isNotEmpty) _videoList(context),
          if (results.isLoadingMore)
            const SliverToBoxAdapter(
              child: Padding(
                padding: EdgeInsets.all(AppSpacing.lg),
                child: Center(child: CircularProgressIndicator()),
              ),
            ),
        ],
      ),
    );
  }

  bool _maybeLoadMore(ScrollNotification notification) {
    if (results.canLoadMore && notification.metrics.extentAfter < 400) {
      actions.onLoadMore();
    }
    return false;
  }

  Widget _creatorsSection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Creators', style: Theme.of(context).textTheme.titleMedium),
        const SizedBox(height: AppSpacing.sm),
        SizedBox(
          height: 104,
          child: ListView.separated(
            scrollDirection: Axis.horizontal,
            itemCount: results.creators.length,
            separatorBuilder: (_, __) => const SizedBox(width: AppSpacing.md),
            itemBuilder: (_, index) => _creatorCard(results.creators[index]),
          ),
        ),
        const SizedBox(height: AppSpacing.lg),
      ],
    );
  }

  Widget _creatorCard(ProfileSummary creator) {
    return InkWell(
      onTap: () => actions.onOpenProfile(creator.id),
      borderRadius: BorderRadius.circular(AppRadius.control),
      child: SizedBox(
        width: 88,
        child: Column(
          children: [
            CircleAvatar(radius: 28, child: Text(creator.initials)),
            const SizedBox(height: AppSpacing.xs),
            Text(
              creator.displayName,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }

  Widget _videosHeader(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: AppSpacing.sm),
      child: Row(
        children: [
          Text('Videos', style: Theme.of(context).textTheme.titleMedium),
          const Spacer(),
          FilledButton.tonalIcon(
            key: const Key('open-in-feed'),
            onPressed: actions.onOpenFeed,
            icon: const Icon(Icons.play_arrow_rounded),
            label: const Text('Open in feed'),
          ),
        ],
      ),
    );
  }

  Widget _videoList(BuildContext context) {
    return SliverList.separated(
      itemCount: results.videos.length,
      separatorBuilder: (_, __) => const SizedBox(height: AppSpacing.sm),
      itemBuilder: (context, index) =>
          _videoTile(context, results.videos[index]),
    );
  }

  Widget _videoTile(BuildContext context, VideoPost post) {
    return ListTile(
      tileColor: Theme.of(context).colorScheme.surface,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppRadius.control),
      ),
      leading: const Icon(Icons.videocam_rounded),
      title: Text(post.caption, maxLines: 2, overflow: TextOverflow.ellipsis),
      subtitle: Text(
        post.creator.displayName,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      trailing: const Icon(Icons.chevron_right),
      onTap: actions.onOpenFeed,
    );
  }
}
