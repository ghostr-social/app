package app.ghostr

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File
import java.util.ArrayDeque

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareIntentIdentityTest {
    @Test
    fun recreationRestoresOneCaptureButAReusedUriStartsANewCapture() {
        val ids = ArrayDeque(listOf("capture-one", "capture-two"))
        val capture = IncomingVideoShareIntentCapture(ids::removeFirst)
        val firstCapture = capture.new(shareIntent())
        val restoredCapture = capture.restore(shareIntent(), firstCapture.savedId)
        val freshCapture = capture.new(shareIntent())
        assertEquals(firstCapture.captureId, restoredCapture.captureId)
        assertNotEquals(firstCapture.captureId, freshCapture.captureId)
        val provider = ShareTestProvider("first".toByteArray(), "video/mp4")
        val context = shareTestContext(provider)

        val first = openBridge(context, firstCapture)
        val firstPath = first.second.invoke("takePendingVideo").path()
        first.first.dispose()
        provider.replaceContent("second".toByteArray())
        val restored = openBridge(context, restoredCapture)
        assertEquals(firstPath, restored.second.invoke("takePendingVideo").path())
        assertEquals(1, provider.openCount)
        restored.first.dispose()

        val fresh = openBridge(context, freshCapture)
        val freshPath = fresh.second.invoke("takePendingVideo").path()
        assertNotEquals(firstPath, freshPath)
        assertArrayEquals("second".toByteArray(), File(freshPath).readBytes())
        assertEquals(2, provider.openCount)
        fresh.second.invoke("releaseVideo", freshPath)
        fresh.first.dispose()
    }

    private fun openBridge(
        context: android.content.Context,
        captured: CapturedIncomingVideoIntent,
    ): Pair<IncomingVideoShareBridge, TestBinaryMessenger> {
        val messenger = TestBinaryMessenger()
        val bridge = IncomingVideoShareBridge(context, messenger, captured, 1L) {}
        return bridge to messenger
    }

    private fun Any?.path(): String = (this as Map<*, *>)["path"] as String
}
