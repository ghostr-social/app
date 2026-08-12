package app.ghostr

import io.flutter.plugin.common.FlutterException
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareBridgeFailureTest {
    @Test
    fun reportsSafeFailuresAndRejectsMalformedChannelCalls() {
        val context = RuntimeEnvironment.getApplication()
        val messenger = TestBinaryMessenger()
        val acknowledged = mutableListOf<Long>()
        val invalid = capturedShareIntent(
            uri = android.net.Uri.parse("file:///private/video.mp4"),
        )
        val bridge = IncomingVideoShareBridge(
            context,
            messenger,
            invalid,
            4L,
            acknowledged::add,
        )

        val failure = assertThrows(FlutterException::class.java) {
            messenger.invoke("takePendingVideo")
        }
        assertEquals("incoming_video_unavailable", failure.code)
        assertEquals(listOf(4L), acknowledged)
        assertThrows(FlutterException::class.java) {
            messenger.invoke("acknowledgeVideo", null)
        }
        context.getSharedPreferences("incoming_video_share", 0).edit()
            .putString("owned_paths", "corrupt")
            .commit()
        assertThrows(FlutterException::class.java) {
            messenger.invoke("releaseVideo", "/not/an/owned/video.mp4")
        }
        assertTrue(messenger.isNotImplemented("unsupportedMethod"))
        bridge.dispose()
    }
}
