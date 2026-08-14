package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.EndDayAction
import com.adiyutant.app.domain.model.PlannerEvent
import com.adiyutant.app.domain.model.ScheduleMode
import com.adiyutant.app.domain.model.SchedulePurpose
import com.adiyutant.app.domain.model.ScheduleRequest
import com.adiyutant.app.domain.model.SheetState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.time.LocalDate
import java.time.LocalTime

class SingleEntityRuleTest {

    private val today = SEED_TODAY
    private val d1 = today.plusDays(1)

    private fun scheduleRequest(taskId: String, initial: LocalDate = today) =
        ScheduleRequest(
            mode = ScheduleMode.SCHEDULE,
            taskId = taskId,
            sourceDate = today,
            initialDate = initial,
            lockDate = false,
            purpose = SchedulePurpose.PLAIN,
        )

    @Test
    fun scheduledTaskNotInDateOnlyNorBacklog() {
        val (core, _) = plannerCoreAt(msAt(today, 9, 0))
        var state = seedState(today)
        assertTrue(dateOnlyTasksOn(state, today).any { it.taskId == "tu1" })

        val slot = firstFreeSlot(state, today, 90)
        state = core.reduce(state, PlannerEvent.ScheduleConfirmed(scheduleRequest("tu1"), today, slot.first, slot.second))

        assertTrue(dateOnlyTasksOn(state, today).none { it.taskId == "tu1" })
        assertNull(state.tasks.single { it.taskId == "tu1" }.plannedDate)
        assertEquals(1, state.blocks.count { it.taskId == "tu1" })
        val groups = backlogGroups(state)
        assertTrue(groups.noDate.none { it.taskId == "tu1" })
        assertTrue(groups.hasDate.none { it.taskId == "tu1" })
        assertTrue(groups.postponed.none { it.taskId == "tu1" })
    }

    @Test
    fun toBacklogRemovesBlockAndTaskGoesToNoDate() {
        val (core, _) = plannerCoreAt(msAt(today, 18, 0))
        var state = seedState(today)
        val blockId = blockIdFor("t4", today)
        state = core.reduce(state, PlannerEvent.EndDayTaskHandled(blockId, EndDayAction.TO_BACKLOG, null))
        assertTrue(state.blocks.none { it.blockId == blockId })
        val task = state.tasks.single { it.taskId == "t4" }
        assertNull(task.plannedDate)
        assertEquals(false, task.postponed)
        assertTrue(backlogGroups(state).noDate.any { it.taskId == "t4" })
    }

    @Test
    fun backlogExcludesTasksWithBlocks() {
        val groups = backlogGroups(seedState(today))
        for (i in 1..16) {
            val id = "t$i"
            assertTrue("scheduled $id leaked into noDate", groups.noDate.none { it.taskId == id })
            assertTrue("scheduled $id leaked into hasDate", groups.hasDate.none { it.taskId == id })
            assertTrue("scheduled $id leaked into postponed", groups.postponed.none { it.taskId == id })
        }
        assertEquals(listOf("tu4"), groups.noDate.map { it.taskId })
        assertEquals(setOf("tu1", "tu2", "tu3", "tu5"), groups.hasDate.map { it.taskId }.toSet())
        assertEquals(listOf("tu6"), groups.postponed.map { it.taskId })
    }

    @Test
    fun postponedImpliesNoPlannedDate() {
        val state = seedState(today)
        val postponed = state.tasks.filter { it.postponed }
        assertTrue(postponed.isNotEmpty())
        assertTrue(postponed.all { it.plannedDate == null })
    }

    @Test
    fun postponedReturnedMovesToHasDateToday() {
        val (core, _) = plannerCoreAt(msAt(today, 9, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.PostponedReturned("tu6"))
        val task = state.tasks.single { it.taskId == "tu6" }
        assertEquals(false, task.postponed)
        assertEquals(today, task.plannedDate)
        assertTrue(dateOnlyTasksOn(state, today).any { it.taskId == "tu6" })
    }

    @Test
    fun noDuplicateTaskIdsAfterOperationSeries() {
        val (core, _) = plannerCoreAt(msAt(today, 9, 0))
        var state = seedState(today)

        val slot = firstFreeSlot(state, today, 60)
        state = core.reduce(state, PlannerEvent.ScheduleConfirmed(scheduleRequest("tu1"), today, slot.first, slot.second))
        val blockId = state.blocks.single { it.taskId == "tu1" }.blockId

        val resched = ScheduleRequest(
            mode = ScheduleMode.RESCHEDULE,
            blockId = blockId,
            sourceDate = today,
            initialDate = d1,
            lockDate = false,
            purpose = SchedulePurpose.PLAIN,
        )
        val slot2 = firstFreeSlot(state, d1, 60)
        state = core.reduce(state, PlannerEvent.ScheduleConfirmed(resched, d1, slot2.first, slot2.second))
        assertEquals(1, state.blocks.count { it.taskId == "tu1" })

        state = core.reduce(state, PlannerEvent.EndDayTaskHandled(state.blocks.single { it.taskId == "tu1" }.blockId, EndDayAction.TO_BACKLOG, null))
        assertEquals(1, state.tasks.count { it.taskId == "tu1" })
        assertEquals(0, state.blocks.count { it.taskId == "tu1" })
    }
}
