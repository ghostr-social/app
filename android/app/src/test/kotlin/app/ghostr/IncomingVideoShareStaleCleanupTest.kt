package app.ghostr

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config
import java.io.File

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareStaleCleanupTest {
    @Test
    fun preservesTheDurablePendingVideoWhileDeletingOldOrphans() {
        val context = RuntimeEnvironment.getApplication()
        context.cacheDir.mkdirs()
        val active = ownedFile(context.cacheDir, "active")
        val orphan = ownedFile(context.cacheDir, "orphan")
        IncomingVideoShareStore(context, IncomingVideoShareCache(context)).save(
            IncomingVideoShare(
                active.path,
                "shared.mp4",
                "video/mp4",
                IncomingVideoShareCaptureId("capture"),
            ),
        )

        IncomingVideoShareReceiver(context).deleteStaleCacheFiles()

        assertTrue(active.exists())
        assertFalse(orphan.exists())
        IncomingVideoShareReceiver(context).release(active.path)
    }

    private fun ownedFile(cacheDirectory: File, marker: String): File {
        return File.createTempFile("incoming-video-$marker-", ".mp4", cacheDirectory).apply {
            writeText(marker)
            setLastModified(0L)
        }
    }
}
