// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#if canImport(video_player_avfoundation_objc)
  import video_player_avfoundation_objc
#endif

struct WarpPreparedPlayerItem {
  let item: FVPAVPlayerItem
  let attemptToken: String?
}
