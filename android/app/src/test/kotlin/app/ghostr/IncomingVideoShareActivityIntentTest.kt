package app.ghostr

import android.content.Intent
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.util.ArrayDeque

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareActivityIntentTest {
    @Test
    fun eachWarmShareGetsANewIdentityAndGeneration() {
        val ids = ArrayDeque(listOf("first", "second"))
        val lifecycle = IncomingVideoShareActivityLifecycle(
            Intent(Intent.ACTION_MAIN),
            null,
            IncomingVideoShareIntentCapture(ids::removeFirst),
        )

        val first = lifecycle.receive(shareIntent())
        val second = lifecycle.receive(shareIntent())
        val launcher = lifecycle.receive(Intent(Intent.ACTION_MAIN))

        assertEquals("first", first.intent.savedId)
        assertEquals(1L, first.generation)
        assertEquals("second", second.intent.savedId)
        assertEquals(2L, second.generation)
        assertNull(launcher.intent.captureId)
        assertEquals(2L, launcher.generation)
    }
}
