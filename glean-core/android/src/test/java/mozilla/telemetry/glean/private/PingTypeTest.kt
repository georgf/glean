/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package mozilla.telemetry.glean.private

import mozilla.telemetry.glean.Glean
import mozilla.telemetry.glean.checkPingSchema
import mozilla.telemetry.glean.getContextWithMockedInfo
import mozilla.telemetry.glean.getMockWebServer
import mozilla.telemetry.glean.getWorkerStatus
import mozilla.telemetry.glean.resetGlean
import mozilla.telemetry.glean.scheduler.PingUploadWorker
import mozilla.telemetry.glean.triggerWorkManager
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.util.concurrent.TimeUnit

@RunWith(RobolectricTestRunner::class)
class PingTypeTest {

    @Before
    fun setUp() {
        resetGlean()
    }

    @Test
    fun `test sending of custom pings`() {
        val server = getMockWebServer()

        val customPing = PingType(
            name = "custom",
            includeClientId = true
        )

        val counter = CounterMetricType(
            disabled = false,
            category = "test",
            lifetime = Lifetime.Ping,
            name = "counter",
            sendInPings = listOf("custom")
        )

        resetGlean(getContextWithMockedInfo(), Glean.configuration.copy(
            serverEndpoint = "http://" + server.hostName + ":" + server.port,
            logPings = true
        ))

        counter.add()
        assertTrue(counter.testHasValue())

        customPing.send()
        // Trigger worker task to upload the pings in the background
        triggerWorkManager()

        val request = server.takeRequest(20L, TimeUnit.SECONDS)
        val docType = request.path.split("/")[3]
        assertEquals("custom", docType)

        val pingJson = JSONObject(request.body.readUtf8())
        assertNotNull(pingJson.getJSONObject("client_info")["client_id"])
        checkPingSchema(pingJson)
    }

    @Test
    fun `test sending of custom pings without client_id`() {
        val server = getMockWebServer()

        val customPing = PingType(
            name = "custom",
            includeClientId = false
        )

        val counter = CounterMetricType(
            disabled = false,
            category = "test",
            lifetime = Lifetime.Ping,
            name = "counter",
            sendInPings = listOf("custom")
        )

        resetGlean(getContextWithMockedInfo(), Glean.configuration.copy(
            serverEndpoint = "http://" + server.hostName + ":" + server.port,
            logPings = true
        ))

        counter.add()
        assertTrue(counter.testHasValue())

        customPing.send()
        // Trigger worker task to upload the pings in the background
        triggerWorkManager()

        val request = server.takeRequest(20L, TimeUnit.SECONDS)
        val docType = request.path.split("/")[3]
        assertEquals("custom", docType)

        val pingJson = JSONObject(request.body.readUtf8())
        assertNull(pingJson.getJSONObject("client_info").opt("client_id"))
        checkPingSchema(pingJson)
    }

    @Test
    fun `Sending a ping with an unknown name is a no-op`() {
        val server = getMockWebServer()

        val counter = CounterMetricType(
            disabled = false,
            category = "test",
            lifetime = Lifetime.Ping,
            name = "counter",
            sendInPings = listOf("unknown")
        )

        resetGlean(getContextWithMockedInfo(), Glean.configuration.copy(
            serverEndpoint = "http://" + server.hostName + ":" + server.port,
            logPings = true
        ))

        counter.add()
        assertTrue(counter.testHasValue())

        Glean.sendPingsByName(listOf("unknown"))

        assertFalse("We shouldn't have any pings scheduled",
            getWorkerStatus(PingUploadWorker.PING_WORKER_TAG).isEnqueued
        )
    }

    @Test
    fun `Registry should contain built-in pings`() {
        assertTrue(Glean.testHasPingType("metrics"))
        assertTrue(Glean.testHasPingType("events"))
        assertTrue(Glean.testHasPingType("baseline"))
    }
}
