import 'package:flutter/material.dart';

class VideoDescription extends StatelessWidget {
  final String username;
  final String videoTitle;
  final String songInfo;

  const VideoDescription({
    super.key,
    required this.username,
    required this.videoTitle,
    required this.songInfo,
  });

  @override
  Widget build(BuildContext context) {
    // Remove `height: 120.0` so the child can expand or shrink properly
    return Expanded(
      child: Container(
        padding: const EdgeInsets.only(left: 20.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.end,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '@$username',
              style: const TextStyle(
                fontSize: 16,
                color: Colors.white,
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 7),
            Text(
              videoTitle,
              style: const TextStyle(
                fontSize: 16,
                color: Colors.white,
              ),
            ),
            const SizedBox(height: 7),
            Row(
              children: [
                const Icon(
                  Icons.music_note,
                  size: 15.0,
                  color: Colors.white,
                ),
                Text(
                  songInfo,
                  style: const TextStyle(color: Colors.white, fontSize: 14.0),
                )
              ],
            ),
            const SizedBox(height: 10),
          ],
        ),
      ),
    );
  }
}
