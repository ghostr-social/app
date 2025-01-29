import 'package:flutter/material.dart';
import 'package:get_it/get_it.dart';
import 'package:video_player/video_player.dart';

import '../../../data/video.dart';
import '../../../widgets/actions_toolbar.dart';
import '../../../widgets/video_description.dart';
import '../../feed_viewmodel.dart';

Widget videoCard(Video video, int index) {
  final feedViewModel = GetIt.instance<FeedViewModel>();
  final controller = feedViewModel.controllerPool[index];
  final isActive = (index == feedViewModel.currentVideoIndex);
  final name = video.user.name ?? video.user.npub ?? "user";

  // If we have a preloaded controller & it's initialized, display the paused frame.
  if (controller != null && controller.value.isInitialized) {
    return GestureDetector(
      onTap: () {
        if (isActive) {
          // Only let the active video play/pause on tap.
          if (controller.value.isPlaying) {
            controller.pause();
          } else {
            controller.play();
          }
        }
      },
      child: Stack(
        children: [
          SizedBox.expand(
            child: FittedBox(
              fit: BoxFit.cover,
              child: SizedBox(
                width: controller.value.size.width,
                height: controller.value.size.height,
                child: VideoPlayer(controller),
              ),
            ),
          ),
          // The overlays you already have:
          Align(
            alignment: Alignment.bottomCenter,
            child:       Column(
              mainAxisAlignment: MainAxisAlignment.end,
              children: <Widget>[
                Row(
                  mainAxisSize: MainAxisSize.max,
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: <Widget>[
                    VideoDescription(
                      username: name,
                      videoTitle: video.videoTitle,
                      songInfo: video.songName,
                    ),
                    ActionsToolbar(
                      video.likes,
                      video.comments,
                      video.user.profilePicture ?? "",
                    ),
                  ],
                ),
                const SizedBox(height: 20),
              ],
            ),
          ),
        ],
      ),
    );
  } else {
    // If we *really* have no controller or not initialized, show placeholder
    return Container(
      color: Colors.black,
      alignment: Alignment.center,
      child: const Text(
        "Loading / Paused",
        style: TextStyle(color: Colors.white),
      ),
    );
  }
}
