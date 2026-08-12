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
class IncomingVideoShareCacheOwnershipTest {
    @Test
    fun deletesOnlyPrefixedFilesDirectlyInsideTheAppCache() {
        val context = RuntimeEnvironment.getApplication()
        val cache = IncomingVideoShareCache(context)
        val owned = File.createTempFile("incoming-video-", ".mp4", context.cacheDir)
        val outside = File.createTempFile("incoming-video-", ".mp4")

        assertTrue(cache.deleteOwnedFile(owned.path))
        assertFalse(owned.exists())
        assertFalse(cache.deleteOwnedFile(outside.path))
        assertTrue(outside.exists())
        outside.delete()
    }
}
