package com.adiyutant.app.core.fake

import com.adiyutant.app.core.CorePort
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class FakeCorePortTest {

    private val port: CorePort = FakeCorePort()

    @Test
    fun todaySnapshotHasSampleValues() = runBlocking {
        val snapshot = port.getTodaySnapshot()
        assertEquals("focus", snapshot.mode)
        assertTrue(snapshot.hasMorningCheckIn)
        assertTrue(snapshot.hasPlan)
        assertEquals(3, snapshot.planItemCount)
    }

    @Test
    fun currentActivityIsScheduledTask() = runBlocking {
        val activity = port.getCurrentActivity()
        assertEquals("scheduled_task", activity.activityKind)
        assertEquals("Глубокая работа", activity.activeTask)
    }
}
