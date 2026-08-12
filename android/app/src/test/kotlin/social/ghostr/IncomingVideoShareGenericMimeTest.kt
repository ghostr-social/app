package social.ghostr

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareGenericMimeTest {
    @Test
    fun resolvesProviderMimeAndCopiesGrantedContent() {
        val bytes = "video-bytes".toByteArray()
        val context = shareTestContext(
            ShareTestProvider(bytes, "video/mp4", "WhatsApp Video.mp4"),
        )
        val receiver = IncomingVideoShareReceiver(context)

        assertEquals("video/mp4", context.contentResolver.getType(SHARE_URI))
        assertArrayEquals(
            bytes,
            context.contentResolver.openInputStream(SHARE_URI)!!.use { it.readBytes() },
        )

        val share = receiver.receive(capturedShareIntent())!!

        assertEquals("video/mp4", share.mimeType)
        assertEquals("WhatsApp Video.mp4", share.label)
        assertEquals(IncomingVideoShareCaptureId("test-capture"), share.sourceKey)
        assertArrayEquals(bytes, File(share.path).readBytes())
        receiver.release(share.path)
        assertFalse(File(share.path).exists())
    }
}
