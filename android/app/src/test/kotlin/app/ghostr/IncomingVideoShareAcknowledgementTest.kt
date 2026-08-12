package app.ghostr

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareAcknowledgementTest {
    @Test
    fun acknowledgedFileRemainsOwnedByDartUntilExplicitRelease() {
        val firstBytes = "first-video".toByteArray()
        val secondBytes = "second-video".toByteArray()
        val provider = ShareTestProvider(firstBytes, "video/mp4")
        val context = shareTestContext(provider)
        val messenger = TestBinaryMessenger()
        val captured = IncomingVideoShareIntentCapture { "capture-one" }.new(shareIntent())
        val bridge = IncomingVideoShareBridge(context, messenger, captured, 1L) {}

        val first = messenger.invoke("takePendingVideo") as Map<*, *>
        val firstPath = first["path"] as String
        messenger.invoke("acknowledgeVideo", firstPath)

        assertNull(IncomingVideoShareReceiver(context).restorePending())
        assertArrayEquals(firstBytes, File(firstPath).readBytes())
        provider.replaceContent(secondBytes)
        val secondCapture = IncomingVideoShareIntentCapture { "capture-two" }
            .new(shareIntent())
        bridge.receive(secondCapture, 2L)
        val second = messenger.invoke("takePendingVideo") as Map<*, *>
        val secondPath = second["path"] as String
        assertArrayEquals(firstBytes, File(firstPath).readBytes())
        assertArrayEquals(secondBytes, File(secondPath).readBytes())

        messenger.invoke("releaseVideo", firstPath)
        messenger.invoke("releaseVideo", secondPath)
        assertFalse(File(firstPath).exists())
        assertFalse(File(secondPath).exists())
        bridge.dispose()
    }
}
