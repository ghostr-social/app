package app.ghostr

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config
import java.io.File

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareStoreTest {
    @Test
    fun restoresCopiedMetadataAcrossReceiverInstancesUntilRelease() {
        val context = RuntimeEnvironment.getApplication()
        val cache = IncomingVideoShareCache(context)
        val store = IncomingVideoShareStore(context, cache)
        store.clear()
        val file = File.createTempFile("incoming-video-", ".mp4", context.cacheDir)
        file.writeText("bytes")
        val share = IncomingVideoShare(
            file.path,
            "shared.mp4",
            "video/mp4",
            IncomingVideoShareCaptureId("capture"),
        )

        store.save(share)

        assertEquals(share, IncomingVideoShareStore(context, cache).load())
        store.release(share.path)
        assertNull(store.load())
        file.delete()
    }
}
