package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.BlockAccent
import com.adiyutant.app.domain.model.DoElseHandling
import com.adiyutant.app.domain.model.DoElseKind
import com.adiyutant.app.domain.model.DoElseSelection
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

class ScheduleSheetContractTest {

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

    private fun rescheduleRequest(blockId: String, lockDate: Boolean = false) =
        ScheduleRequest(
            mode = ScheduleMode.RESCHEDULE,
            blockId = blockId,
            sourceDate = today,
            initialDate = today,
            lockDate = lockDate,
            purpose = SchedulePurpose.PLAIN,
        )

    @Test
    fun scheduleCreatesDeterministicBlockAndClearsPlannedDate() {
        val (core, _) = plannerCoreAt(msAt(today, 9, 0))
        var state = seedState(today)
        val req = scheduleRequest("tu3")
        state = core.reduce(state, PlannerEvent.OpenScheduleSheet(req))
        assertEquals(SheetState.ScheduleSheet(req), state.sheet)

        val slot = firstFreeSlot(state, today, 15)
        state = core.reduce(state, PlannerEvent.ScheduleConfirmed(req, today, slot.first, slot.second))
        val block = state.blocks.single { it.taskId == "tu3" }
        assertEquals("s_tu3_${yyyyMMdd(today)}", block.blockId)
        assertEquals(0, block.progressPercent)
        assertEquals(BlockAccent.ORANGE, block.accent)
        assertEquals(slot.first, block.start)
        assertNull(state.tasks.single { it.taskId == "tu3" }.plannedDate)
    }

    @Test
    fun scheduleClearsPlannedDateRegardlessOfTargetDate() {
        val (core, _) = plannerCoreAt(msAt(today, 9, 0))
        var state = seedState(today)
        val req = scheduleRequest("tu3")
        val target = today.plusDays(3)
        state = core.reduce(state, PlannerEvent.ScheduleConfirmed(req, target, LocalTime.of(10, 0), LocalTime.of(10, 15)))
        assertNull(state.tasks.single { it.taskId == "tu3" }.plannedDate)
        assertEquals(target, state.blocks.single { it.taskId == "tu3" }.date)
    }

    @Test
    fun sameDateReschedulePreservesBlockId() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        val block = state.blocks.single { it.taskId == "t3" }
        val req = rescheduleRequest(block.blockId, lockDate = true)
        state = core.reduce(state, PlannerEvent.ScheduleConfirmed(req, today, LocalTime.of(13, 0), LocalTime.of(14, 0)))
        val updated = state.blocks.single { it.blockId == block.blockId }
        assertEquals(block.blockId, updated.blockId)
        assertEquals("t3", updated.taskId)
        assertEquals(today, updated.date)
        assertEquals(LocalTime.of(13, 0), updated.start)
        assertEquals(LocalTime.of(14, 0), updated.end)
        assertEquals(45, updated.progressPercent)
    }

    @Test
    fun crossDateReschedulePreservesBlockIdAndTaskId() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        val block = state.blocks.single { it.taskId == "t3" }
        val req = rescheduleRequest(block.blockId)
        val slot = firstFreeSlot(state, d1, computeBlockDuration(block))
        state = core.reduce(state, PlannerEvent.ScheduleConfirmed(req, d1, slot.first, slot.second))
        assertEquals(0, state.blocks.count { it.blockId == block.blockId && it.date == today })
        assertEquals(1, state.blocks.count { it.taskId == "t3" })
        val moved = state.blocks.single { it.taskId == "t3" }
        assertEquals(block.blockId, moved.blockId)
        assertEquals(d1, moved.date)
        assertEquals(block.progressPercent, moved.progressPercent)
        assertEquals(block.accent, moved.accent)
        assertEquals("t3", moved.taskId)
    }

    @Test
    fun rejectsOutOfRangeTimes() {
        val (core, _) = plannerCoreAt(msAt(today, 9, 0))
        val req = rescheduleRequest(blockIdFor("t3", today), lockDate = true)
        val state = seedState(today)
        val before = state

        // start before 09:00
        assertEquals(before, core.reduce(state, PlannerEvent.ScheduleConfirmed(req, today, LocalTime.of(8, 30), LocalTime.of(9, 30))))
        // end after 19:00
        assertEquals(before, core.reduce(state, PlannerEvent.ScheduleConfirmed(req, today, LocalTime.of(18, 0), LocalTime.of(19, 30))))
        // end <= start
        assertEquals(before, core.reduce(state, PlannerEvent.ScheduleConfirmed(req, today, LocalTime.of(10, 0), LocalTime.of(10, 0))))
    }

    @Test
    fun rejectsOverlappingSlot() {
        val (core, _) = plannerCoreAt(msAt(today, 9, 0))
        var state = seedState(today)
        val req = rescheduleRequest(blockIdFor("t3", today))
        state = core.reduce(state, PlannerEvent.ScheduleConfirmed(req, today, LocalTime.of(9, 15), LocalTime.of(10, 15)))
        val unchanged = state.blocks.single { it.taskId == "t3" }
        assertEquals(LocalTime.of(11, 30), unchanged.start)
        assertEquals(LocalTime.of(13, 0), unchanged.end)
    }

    @Test
    fun ownBlockExcludedFromOverlapAtReschedule() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        val req = rescheduleRequest(blockIdFor("t3", today), lockDate = true)
        state = core.reduce(state, PlannerEvent.ScheduleConfirmed(req, today, LocalTime.of(12, 0), LocalTime.of(12, 30)))
        val updated = state.blocks.single { it.taskId == "t3" }
        assertEquals(LocalTime.of(12, 0), updated.start)
        assertEquals(LocalTime.of(12, 30), updated.end)
    }

    @Test
    fun validateEnforcesWorkdayBoundsAndOverlap() {
        val existing = blocksOn(seedState(today), today)
        assertEquals(
            ValidationError.OUT_OF_RANGE,
            ScheduleValidator.validate(LocalTime.of(8, 0), LocalTime.of(9, 0), existing, null),
        )
        assertEquals(
            ValidationError.OUT_OF_RANGE,
            ScheduleValidator.validate(LocalTime.of(9, 0), LocalTime.of(9, 0), existing, null),
        )
        assertEquals(
            ValidationError.OVERLAP,
            ScheduleValidator.validate(LocalTime.of(9, 0), LocalTime.of(9, 30), existing, null),
        )
        assertEquals(
            null,
            ScheduleValidator.validate(LocalTime.of(10, 0), LocalTime.of(10, 15), existing, null),
        )
    }

    @Test
    fun freeSlotsSkipConflictsAndOwnBlock() {
        val state = seedState(today)
        val existing = blocksOn(state, today)
        val slots = ScheduleValidator.freeSlots(existing, 60, blockIdFor("t3", today))
        assertTrue(slots.contains(LocalTime.of(11, 30) to LocalTime.of(12, 30)))
        assertTrue(slots.contains(LocalTime.of(17, 30) to LocalTime.of(18, 30)))
        assertTrue(slots.none { it.first == LocalTime.of(9, 0) && it.second == LocalTime.of(10, 0) })
    }

    @Test
    fun cancelIsLossless() {
        val (core, _) = plannerCoreAt(msAt(today, 9, 0))
        var state = seedState(today)
        val beforeTasks = state.tasks
        val beforeBlocks = state.blocks
        val req = scheduleRequest("tu4")
        state = core.reduce(state, PlannerEvent.OpenScheduleSheet(req))
        state = core.reduce(state, PlannerEvent.ScheduleCancelled)
        assertNull(state.sheet)
        assertEquals(beforeTasks, state.tasks)
        assertEquals(beforeBlocks, state.blocks)
        assertTrue(state.factSessions.isEmpty())
        assertNull(state.activeSession)
    }

    @Test
    fun cancelDoElseScheduleKeepsOriginalSessionAndSchedule() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(blockIdFor("t3", today)))
        clock.advance(10_000)
        state = core.reduce(state, PlannerEvent.DoElseRequested(blockIdFor("t3", today)))
        state = core.reduce(state, PlannerEvent.DoElseConfirmed(
            DoElseSelection(DoElseKind.UNPLANNED, "Перерыв", null),
            DoElseHandling.SHIFT,
        ))
        assertTrue(state.pendingDoElse != null)
        val sheet = state.sheet as SheetState.ScheduleSheet
        assertEquals(SchedulePurpose.DO_ELSE, sheet.request.purpose)
        val originalSession = state.activeSession
        val beforeBlocks = state.blocks

        state = core.reduce(state, PlannerEvent.ScheduleCancelled)
        assertNull(state.pendingDoElse)
        assertNull(state.sheet)
        assertEquals(originalSession, state.activeSession)
        assertEquals(beforeBlocks, state.blocks)
        assertTrue(state.factSessions.isEmpty())
    }
}
