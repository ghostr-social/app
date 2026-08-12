package social.ghostr

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareReleaseAcknowledgementTest {
    @Test
    fun releasingARejectedShareDisarmsItsCapturedActivityIntent() {
        val context = shareTestContext(
            ShareTestProvider("video".toByteArray(), "video/mp4"),
        )
        val messenger = TestBinaryMessenger()
        val acknowledged = mutableListOf<Long>()
        val captured = IncomingVideoShareIntentCapture { "capture" }.new(shareIntent())
        val bridge = IncomingVideoShareBridge(
            context,
            messenger,
            captured,
            7L,
            acknowledged::add,
        )
        val payload = messenger.invoke("takePendingVideo") as Map<*, *>
        val path = payload["path"] as String

        messenger.invoke("releaseVideo", path)

        assertEquals(listOf(7L), acknowledged)
        assertFalse(File(path).exists())
        bridge.dispose()
    }
}
