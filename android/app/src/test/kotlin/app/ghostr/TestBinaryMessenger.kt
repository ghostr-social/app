package app.ghostr

import io.flutter.plugin.common.BinaryMessenger
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.StandardMethodCodec
import org.robolectric.shadows.ShadowLooper
import java.nio.ByteBuffer
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger

internal class TestBinaryMessenger : BinaryMessenger {
    private var handler: BinaryMessenger.BinaryMessageHandler? = null

    override fun send(channel: String, message: ByteBuffer?) = Unit

    override fun send(
        channel: String,
        message: ByteBuffer?,
        callback: BinaryMessenger.BinaryReply?,
    ) {
        callback?.reply(null)
    }

    override fun setMessageHandler(
        channel: String,
        handler: BinaryMessenger.BinaryMessageHandler?,
    ) {
        this.handler = handler
    }

    fun invoke(method: String, arguments: Any? = null): Any? {
        return beginInvoke(method, arguments).await()
    }

    fun isNotImplemented(method: String): Boolean {
        return beginInvoke(method).awaitEnvelope() == null
    }

    fun beginInvoke(method: String, arguments: Any? = null): PendingMethodReply {
        val reply = PendingMethodReply()
        val message = StandardMethodCodec.INSTANCE.encodeMethodCall(
            MethodCall(method, arguments),
        ).apply(ByteBuffer::flip)
        handler!!.onMessage(message, reply::complete)
        return reply
    }
}

internal class PendingMethodReply {
    private val latch = CountDownLatch(1)
    private val replies = AtomicInteger()
    @Volatile private var envelope: ByteBuffer? = null

    val replyCount: Int
        get() = replies.get()

    fun complete(value: ByteBuffer?) {
        envelope = value?.apply(ByteBuffer::flip)
        replies.incrementAndGet()
        latch.countDown()
    }

    fun await(): Any? {
        val value = awaitEnvelope() ?: return null
        return StandardMethodCodec.INSTANCE.decodeEnvelope(value)
    }

    fun awaitEnvelope(): ByteBuffer? {
        repeat(500) {
            ShadowLooper.runUiThreadTasksIncludingDelayedTasks()
            if (latch.await(10, TimeUnit.MILLISECONDS)) return envelope
        }
        error("Timed out waiting for platform reply.")
    }
}
