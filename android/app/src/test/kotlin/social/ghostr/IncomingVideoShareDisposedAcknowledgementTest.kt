package social.ghostr

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareDisposedAcknowledgementTest {
    @Test
    fun acceptedAcknowledgementSettlesAndPreventsRecreatedBridgeReplay() {
        val provider = ShareTestProvider("video".toByteArray(), "video/mp4")
        val context = shareTestContext(provider)
        val captured = IncomingVideoShareIntentCapture { "capture" }.new(shareIntent())
        val messenger = TestBinaryMessenger()
        val acknowledged = mutableListOf<Long>()
        val original = IncomingVideoShareBridge(
            context, messenger, captured, 9L, acknowledged::add,
        )
        val path = (messenger.invoke("takePendingVideo") as Map<*, *>)["path"] as String
        val workerEntered = CountDownLatch(1)
        val releaseWorker = CountDownLatch(1)
        IncomingVideoShareWorker.execute {
            workerEntered.countDown()
            releaseWorker.await(5, TimeUnit.SECONDS)
        }
        assertTrue(workerEntered.await(5, TimeUnit.SECONDS))
        val reply = messenger.beginInvoke("acknowledgeVideo", path)
        original.dispose()
        val replacementMessenger = TestBinaryMessenger()
        val replacement = IncomingVideoShareBridge(
            context, replacementMessenger, captured, 9L, acknowledged::add,
        )

        releaseWorker.countDown()

        assertNull(reply.await())
        assertEquals(1, reply.replyCount)
        assertEquals(listOf(9L), acknowledged)
        assertNull(replacementMessenger.invoke("takePendingVideo"))
        assertEquals(1, provider.openCount)
        assertTrue(File(path).isFile)
        replacementMessenger.invoke("releaseVideo", path)
        replacement.dispose()
    }
}
