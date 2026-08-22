// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package io.flutter.plugins.videoplayer;

import static org.junit.Assert.assertEquals;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyBoolean;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

import androidx.media3.datasource.DefaultHttpDataSource;
import androidx.test.core.app.ApplicationProvider;
import java.util.LinkedHashMap;
import java.util.Map;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.robolectric.RobolectricTestRunner;

@RunWith(RobolectricTestRunner.class)
public final class WarpHttpVideoAssetTest {
  @Test
  public void reservedAttemptHeaderNeverReachesHttpDataSource() {
    Map<String, String> headers = new LinkedHashMap<>();
    headers.put("Authorization", "preserved");
    headers.put(WarpPlaybackAttemptHeaders.HEADER_NAME, "AAAAAAAAAAAAAAAAAAAAAA");
    VideoAsset asset =
        VideoAsset.fromRemoteUrl(
            "https://example.test/video.mp4",
            VideoAsset.StreamingFormat.UNKNOWN,
            headers,
            null);
    DefaultHttpDataSource.Factory factory = mockFactory();

    ((HttpVideoAsset) asset)
        .getMediaSourceFactory(ApplicationProvider.getApplicationContext(), factory);

    verify(factory).setDefaultRequestProperties(Map.of("Authorization", "preserved"));
    assertEquals("AAAAAAAAAAAAAAAAAAAAAA", asset.getWarpPlaybackAttemptToken());
  }

  private static DefaultHttpDataSource.Factory mockFactory() {
    DefaultHttpDataSource.Factory factory = mock(DefaultHttpDataSource.Factory.class);
    when(factory.setUserAgent(any())).thenReturn(factory);
    when(factory.setAllowCrossProtocolRedirects(anyBoolean())).thenReturn(factory);
    when(factory.setDefaultRequestProperties(any())).thenReturn(factory);
    return factory;
  }
}
