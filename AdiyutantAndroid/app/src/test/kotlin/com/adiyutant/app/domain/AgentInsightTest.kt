package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.AgentInsightKind
import com.adiyutant.app.domain.model.BlockAccent
import com.adiyutant.app.domain.model.CheckIn
import com.adiyutant.app.domain.model.Mood
import com.adiyutant.app.domain.model.Priority
import com.adiyutant.app.domain.model.ScheduledBlock
import com.adiyutant.app.domain.model.Task
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.time.LocalDate
import java.time.LocalTime

class AgentInsightTest {

    private val today = SEED_TODAY

    private fun task(taskId: String, title: String = "Задача", durationMin: Int = 60) =
        Task(taskId, title, "", durationMin, Priority.MEDIUM, null, false)

    private fun block(taskId: String, start: String, end: String, progress: Int = 0, date: LocalDate = today) =
        ScheduledBlock(
            blockId = blockIdFor(taskId, date),
            taskId = taskId,
            date = date,
            start = LocalTime.parse(start),
            end = LocalTime.parse(end),
            progressPercent = progress,
            accent = BlockAccent.BLUE,
        )

    @Test
    fun todayOverloadTakesPriority() {
        val blocks = listOf(block("a", "09:00", "18:30"))
        val state = lightState(
            blocks = blocks,
            tasks = listOf(task("a", "A")),
            checkIn = CheckIn(0, 1, Mood.OK, null, 0),
        )
        assertEquals(AgentInsightKind.TODAY_OVERLOAD, todayAgentInsight(state, msAt(today, 12, 0))?.kind)
    }

    @Test
    fun lowEnergyUsesFirstUnfinishedBlockByStart() {
        val blocks = listOf(
            block("b", "12:00", "13:00"),
            block("a", "10:00", "11:00", progress = 100),
            block("c", "09:00", "10:00"),
        )
        val tasks = listOf(task("a", "A"), task("b", "B"), task("c", "C"))
        val state = lightState(blocks = blocks, tasks = tasks, checkIn = CheckIn(0, 1, Mood.OK, null, 0))
        val insight = todayAgentInsight(state, msAt(today, 12, 0))
        assertEquals(AgentInsightKind.TODAY_LOW_ENERGY, insight?.kind)
        assertEquals("C", insight?.blockTitle)
    }

    @Test
    fun noTodayInsightWithoutOverloadOrLowEnergy() {
        val blocks = listOf(block("a", "10:00", "11:00"))
        val state = lightState(blocks = blocks, tasks = listOf(task("a", "A")), checkIn = CheckIn(2, 2, Mood.OK, null, 0))
        assertNull(todayAgentInsight(state, msAt(today, 12, 0)))
    }

    @Test
    fun noTodayInsightWhenEnergyNotZero() {
        val blocks = listOf(block("a", "10:00", "11:00"))
        val state = lightState(blocks = blocks, tasks = listOf(task("a", "A")), checkIn = CheckIn(1, 2, Mood.OK, null, 0))
        assertNull(todayAgentInsight(state, msAt(today, 12, 0)))
    }

    @Test
    fun noTodayInsightWhenNoCheckIn() {
        val blocks = listOf(block("a", "10:00", "11:00"))
        val state = lightState(blocks = blocks, tasks = listOf(task("a", "A")))
        assertNull(todayAgentInsight(state, msAt(today, 12, 0)))
    }

    @Test
    fun planOverloadWhenPlannedExceeds480() {
        val blocks = listOf(block("a", "09:00", "18:30"))
        val state = lightState(blocks = blocks, tasks = listOf(task("a", "A")))
        assertEquals(AgentInsightKind.PLAN_OVERLOAD, planAgentInsight(state, msAt(today, 12, 0))?.kind)
    }

    @Test
    fun planPastDuePicksFirstUnfinishedPastEnd() {
        val blocks = listOf(
            block("a", "09:00", "10:00", progress = 100),
            block("b", "10:00", "11:00"),
            block("c", "12:30", "13:30"),
        )
        val tasks = listOf(task("a", "A"), task("b", "B"), task("c", "C"))
        val state = lightState(blocks = blocks, tasks = tasks)
        val insight = planAgentInsight(state, msAt(today, 12, 0))
        assertEquals(AgentInsightKind.PLAN_PAST_DUE, insight?.kind)
        assertEquals("B", insight?.blockTitle)
    }

    @Test
    fun noPlanInsightWhenNothingPastDue() {
        val blocks = listOf(block("a", "12:30", "13:30"))
        val state = lightState(blocks = blocks, tasks = listOf(task("a", "A")))
        assertNull(planAgentInsight(state, msAt(today, 12, 0)))
    }

    @Test
    fun seedTodayNotOverloadedForAgents() {
        val state = seedState(today)
        assertNull(todayAgentInsight(state, msAt(today, 12, 0)))
        assertNull(planAgentInsight(state, msAt(today, 12, 0)))
    }
}
