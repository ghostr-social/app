import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

Future<void> showFeedCardMenu(
  BuildContext context, {
  required VideoPost post,
  required VoidCallback onBlockCreator,
}) {
  return showModalBottomSheet<void>(
    context: context,
    backgroundColor: AppPalette.surface,
    barrierColor: AppPalette.videoSheetBarrier,
    builder: (sheet) => _FeedCardMenu(post: post, onBlock: onBlockCreator),
  );
}

class _FeedCardMenu extends StatelessWidget {
  const _FeedCardMenu({required this.post, required this.onBlock});

  final VideoPost post;
  final VoidCallback onBlock;

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          ListTile(
            leading: const Icon(Icons.block, color: AppPalette.accentRed),
            title: Text('Block ${post.creator.handle}'),
            subtitle: const Text('Hide every video from this creator'),
            onTap: () => _block(context),
          ),
          ListTile(
            leading: const Icon(Icons.close, color: AppPalette.mutedForeground),
            title: const Text('Cancel'),
            onTap: Navigator.of(context).pop,
          ),
        ],
      ),
    );
  }

  void _block(BuildContext context) {
    Navigator.of(context).pop();
    onBlock();
  }
}
