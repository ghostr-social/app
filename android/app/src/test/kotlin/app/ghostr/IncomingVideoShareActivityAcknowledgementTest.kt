package app.ghostr

import android.content.Intent
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareActivityAcknowledgementTest {
    @Test
    fun onlyTheCurrentShareCanResetTheActivityIntent() {
        val lifecycle = IncomingVideoShareActivityLifecycle(
            shareIntent(),
            "capture",
        )
        val active = lifecycle.configureEngine()

        assertNull(lifecycle.acknowledge(active.generation - 1))
        assertEquals("capture", lifecycle.savedCaptureId)

        val launcher = lifecycle.acknowledge(active.generation)

        assertEquals(Intent.ACTION_MAIN, launcher?.action)
        assertNull(lifecycle.savedCaptureId)
        assertNull(lifecycle.acknowledge(active.generation))
    }
}
