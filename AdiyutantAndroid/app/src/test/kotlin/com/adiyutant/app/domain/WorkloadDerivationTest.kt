package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.BlockAccent
import com.adiyutant.app.domain.model.Priority
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class WorkloadDerivationTest {

    private val today = SEED_TODAY

    @Test
    fun workloadComputesPlannedAndWithoutTime() {
        val w = workload(seedState(today), today)
        assertEquals(380, w.plannedMin)
        assertEquals(135, w.withoutTimeMin)
        assertEquals(515, w.totalMin)
        assertEquals(true, w.overload)
    }

    @Test
    fun workloadForTomorrowIsLight() {
        val w = workload(seedState(today), today.plusDays(1))
        assertEquals(240, w.plannedMin)
        assertEquals(15, w.withoutTimeMin)
        assertEquals(255, w.totalMin)
        assertEquals(false, w.overload)
    }

    @Test
    fun kpiCountsTodayProgress() {
        val k = kpi(seedState(today))
        assertEquals(2, k.doneToday)
        assertEquals(6, k.totalToday)
        assertEquals(0, k.topDone)
        assertEquals(3, k.topTotal)
        assertEquals(120, k.doneMin)
        assertEquals(380, k.blockMin)
    }

    @Test
    fun topThreeItemsResolvesToTodayBlocks() {
        val state = seedState(today)
        val items = topThreeItems(state)
        assertEquals(listOf("t3", "t4", "t5"), items.map { it.taskId })
        assertTrue(items.all { it.date == today })
    }

    @Test
    fun accentForMapsProjectsAndPriority() {
        assertEquals(BlockAccent.PURPLE, accentFor(Priority.HIGH, "Обучение"))
        assertEquals(BlockAccent.PURPLE, accentFor(Priority.LOW, "Self Development"))
        assertEquals(BlockAccent.BLUE, accentFor(Priority.HIGH, "Adiyutant"))
        assertEquals(BlockAccent.ORANGE, accentFor(Priority.LOW, "Adiyutant"))
        assertEquals(BlockAccent.BLUE, accentFor(Priority.MEDIUM, "Adiyutant"))
    }
}
