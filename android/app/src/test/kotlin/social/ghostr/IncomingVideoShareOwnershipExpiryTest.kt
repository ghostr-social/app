package social.ghostr

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareOwnershipExpiryTest {
    @Test
    fun restartedReceiverExpiresAcknowledgedFilesButKeepsPendingShare() {
        val provider = ShareTestProvider("first".toByteArray(), "video/mp4")
        val context = shareTestContext(provider)
        val receiver = IncomingVideoShareReceiver(context)
        val accepted = receiver.receive(capturedShareIntent(id = "accepted"))!!
        receiver.acknowledge(accepted.path)
        provider.replaceContent("pending".toByteArray())
        val pending = receiver.receive(capturedShareIntent(id = "pending"))!!
        File(accepted.path).setLastModified(0L)
        File(pending.path).setLastModified(0L)

        val restarted = IncomingVideoShareReceiver(context)
        restarted.deleteStaleCacheFiles()

        assertFalse(File(accepted.path).exists())
        assertTrue(File(pending.path).isFile)
        assertTrue(restarted.restorePending()?.path == pending.path)
        restarted.release(pending.path)
    }
}
