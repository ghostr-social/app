// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package io.flutter.plugins.videoplayer;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.media3.common.Tracks;

final class SelectedTrackId {
  @Nullable
  static String find(@NonNull Tracks tracks, int trackType) {
    int groupIndex = 0;
    for (Tracks.Group group : tracks.getGroups()) {
      String id = selectedInGroup(group, trackType, groupIndex);
      if (id != null) return id;
      groupIndex++;
    }
    return null;
  }

  @Nullable
  private static String selectedInGroup(Tracks.Group group, int trackType, int groupIndex) {
    if (group.getType() != trackType || !group.isSelected()) return null;
    for (int trackIndex = 0; trackIndex < group.length; trackIndex++) {
      if (group.isTrackSelected(trackIndex)) {
        // Keep this format in sync with android_video_player.dart::_parseAndroidTrackId.
        return groupIndex + "_" + trackIndex;
      }
    }
    return null;
  }

  private SelectedTrackId() {}
}
