package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.ActiveKind
import com.adiyutant.app.domain.model.ActiveSession
import com.adiyutant.app.domain.model.AgentInsight
import com.adiyutant.app.domain.model.AgentInsightKind
import com.adiyutant.app.domain.model.BacklogGroups
import com.adiyutant.app.domain.model.BlockAccent
import com.adiyutant.app.domain.model.CheckIn
import com.adiyutant.app.domain.model.DayPhase
import com.adiyutant.app.domain.model.DoElseHandling
import com.adiyutant.app.domain.model.DoElseKind
import com.adiyutant.app.domain.model.DoElseSelection
import com.adiyutant.app.domain.model.EndDayReview
import com.adiyutant.app.domain.model.Ease
import com.adiyutant.app.domain.model.EndDayAction
import com.adiyutant.app.domain.model.HeroState
import com.adiyutant.app.domain.model.Kpi
import com.adiyutant.app.domain.model.Mood
import com.adiyutant.app.domain.model.NotDoneAction
import com.adiyutant.app.domain.model.NoteKind
import com.adiyutant.app.domain.model.PendingDoElse
import com.adiyutant.app.domain.model.PlannerEvent
import com.adiyutant.app.domain.model.PlannerState
import com.adiyutant.app.domain.model.Priority
import com.adiyutant.app.domain.model.QuickNote
import com.adiyutant.app.domain.model.ScheduleMode
import com.adiyutant.app.domain.model.SchedulePurpose
import com.adiyutant.app.domain.model.ScheduleRequest
import com.adiyutant.app.domain.model.ScheduledBlock
import com.adiyutant.app.domain.model.SessionKind
import com.adiyutant.app.domain.model.SheetState
import com.adiyutant.app.domain.model.Task
import com.adiyutant.app.domain.model.WorkSession
import com.adiyutant.app.domain.model.Workload
import java.time.Instant
import java.time.LocalDate
import java.time.LocalTime
import java.time.ZoneId

/**
 * Чистый синхронный reducer (разделы 3, 4, 7, 10, 11 ТЗ).
 *
 * Без корутин и Android: время берётся из [Clock]. Reducer повторно валидирует
 * confirm-данные (R6) и защищает инварианты single-entity / единственной
 * активной сессии / завершённого блока.
 */
class PlannerCore(private val clock: Clock) {

    fun reduce(state: PlannerState, event: PlannerEvent): PlannerState {
        val now = clock.nowMs()
        val today = state.todayDate
        return when (event) {
            is PlannerEvent.CheckInSubmitted -> checkInSubmitted(state, event.energy, event.focus, event.mood, event.obstacle, now)
            is PlannerEvent.DayModeSelected -> state.copy(dayMode = event.mode)
            is PlannerEvent.StartBlock -> startBlock(state, event.blockId, now)
            is PlannerEvent.TopThreeStart -> startBlock(state, event.blockId, now)
            PlannerEvent.PauseSession -> pauseSession(state, now)
            PlannerEvent.ResumeSession -> resumeSession(state, now)
            PlannerEvent.FinishActiveSession -> finishActiveSession(state, now)
            PlannerEvent.BlockEndedStillWorking -> blockEndedStillWorking(state)
            PlannerEvent.BlockEndedNotDone -> blockEndedNotDone(state)
            is PlannerEvent.OpenCompletionSheet -> openCompletionSheet(state, event.blockId)
            is PlannerEvent.CompletionConfirmed -> completionConfirmed(state, event.ease, now)
            is PlannerEvent.NotDoneAction -> notDoneAction(state, event.action, today)
            is PlannerEvent.DoElseRequested -> state.copy(sheet = SheetState.DoElseSheet(event.blockId))
            is PlannerEvent.DoElseConfirmed -> doElseConfirmed(state, event.selection, event.handling, now, today)
            is PlannerEvent.OpenScheduleSheet -> state.copy(sheet = SheetState.ScheduleSheet(event.request))
            is PlannerEvent.ScheduleConfirmed -> scheduleConfirmed(state, event.request, event.date, event.start, event.end, now)
            PlannerEvent.ScheduleCancelled -> cancelSheet(state)
            is PlannerEvent.OpenTaskDetails -> state.copy(sheet = SheetState.TaskDetailsSheet(event.blockId, event.sourceDate))
            is PlannerEvent.TaskDetailsReschedule -> taskDetailsReschedule(state, event.sameDateOnly, today)
            is PlannerEvent.TaskDetailsComplete -> completeFromTaskDetails(state, event.blockId)
            is PlannerEvent.TopThreeComplete -> completeFromTopThree(state, event.blockId)
            PlannerEvent.CloseSheet -> cancelSheet(state)
            is PlannerEvent.QuickNoteAdded -> quickNoteAdded(state, event.text, event.kind, now)
            PlannerEvent.OpenEndOfDay -> state.copy(dayPhase = DayPhase.END_OF_DAY, sheet = SheetState.EndDaySheet)
            is PlannerEvent.EndDayTaskHandled -> endDayTaskHandled(state, event.blockId, event.action, event.targetDate, today, now)
            is PlannerEvent.EndDayEnergySelected -> state.copy(dayEnergy = event.energy)
            PlannerEvent.EndDayFinished -> endDayFinished(state, now)
            is PlannerEvent.PlanModeSelected -> state.copy(planUi = state.planUi.copy(mode = event.mode))
            is PlannerEvent.PlanDateSelected -> state.copy(planUi = state.planUi.copy(selectedDate = event.date))
            is PlannerEvent.ViewMonthShifted -> state.copy(planUi = state.planUi.copy(viewMonth = state.planUi.viewMonth.plusMonths(event.deltaMonths.toLong())))
            is PlannerEvent.PostponedReturned -> postponedReturned(state, event.taskId, today)
        }
    }

    // ── Today: чек-ин, режим, старт/пауза/завершение ──

    private fun checkInSubmitted(
        state: PlannerState,
        energy: Int,
        focus: Int,
        mood: Mood,
        obstacle: String?,
        now: Long,
    ): PlannerState {
        var next = state.copy(
            dayCheckIn = CheckIn(energy = energy, focus = focus, mood = mood, obstacle = obstacle, atMs = now),
            dayEnergy = energy,
        )
        if (state.dayPhase == DayPhase.DAY_NOT_STARTED) {
            next = next.copy(dayPhase = DayPhase.RUNNING)
        }
        return next
    }

    private fun startBlock(state: PlannerState, blockId: String, now: Long): PlannerState {
        if (state.activeSession != null) return state
        val block = state.blocks.firstOrNull { it.blockId == blockId } ?: return state
        if (block.progressPercent == 100) return state
        return state.copy(
            activeSession = ActiveSession(
                kind = ActiveKind.PLANNED,
                taskId = block.taskId,
                blockId = block.blockId,
                parentBlockId = null,
                label = null,
                startedAtMs = now,
                runningSinceMs = now,
                accumulatedMs = 0,
                paused = false,
                plannedStart = block.start,
                plannedEnd = block.end,
            ),
        )
    }

    private fun pauseSession(state: PlannerState, now: Long): PlannerState {
        val s = state.activeSession ?: return state
        if (s.paused) return state
        val runningSince = s.runningSinceMs ?: return state
        return state.copy(
            activeSession = s.copy(
                accumulatedMs = s.accumulatedMs + (now - runningSince),
                runningSinceMs = null,
                paused = true,
            ),
        )
    }

    private fun resumeSession(state: PlannerState, now: Long): PlannerState {
        val s = state.activeSession ?: return state
        if (!s.paused) return state
        return state.copy(activeSession = s.copy(paused = false, runningSinceMs = now))
    }

    private fun finishActiveSession(state: PlannerState, now: Long): PlannerState {
        val s = state.activeSession ?: return state
        return when (s.kind) {
            ActiveKind.PLANNED -> {
                val block = s.blockId?.let { state.blocks.firstOrNull { b -> b.blockId == it } } ?: return state
                state.copy(sheet = SheetState.BlockEndedSheet(block.blockId))
            }
            ActiveKind.UNPLANNED_WORK -> {
                val kind = s.sessionKind ?: SessionKind.UNPLANNED
                val session = WorkSession(
                    sessionId = sessionIdFor(null, s.parentBlockId, s.label, s.startedAtMs),
                    taskId = null,
                    blockId = null,
                    parentBlockId = s.parentBlockId,
                    kind = kind,
                    label = s.label,
                    startedAtMs = s.startedAtMs,
                    endedAtMs = now,
                    durationSec = elapsedMs(s, now) / 1000,
                    ease = null,
                )
                state.copy(activeSession = null, factSessions = state.factSessions + session)
            }
        }
    }

    private fun blockEndedStillWorking(state: PlannerState): PlannerState =
        if (state.sheet is SheetState.BlockEndedSheet) state.copy(sheet = null) else state

    private fun blockEndedNotDone(state: PlannerState): PlannerState {
        val sheet = state.sheet as? SheetState.BlockEndedSheet ?: return state
        return state.copy(activeSession = null, sheet = SheetState.NotDoneSheet(sheet.blockId))
    }

    /**
     * Открывает [SheetState.CompletionSheet] только для блока, который реально
     * можно завершить через [CompletionConfirmed] (симметричный контракт):
     * блок существует, `progressPercent < 100`, активна PLANNED-сессия именно
     * этого блока. Иначе — no-op. Бизнес-данные не мутируются.
     */
    private fun openCompletionSheet(state: PlannerState, blockId: String): PlannerState {
        val block = state.blocks.firstOrNull { it.blockId == blockId } ?: return state
        if (block.progressPercent == 100) return state
        val s = state.activeSession ?: return state
        if (s.kind != ActiveKind.PLANNED || s.blockId != blockId) return state
        return state.copy(sheet = SheetState.CompletionSheet(blockId))
    }

    private fun completionConfirmed(state: PlannerState, ease: Ease, now: Long): PlannerState {
        val s = state.activeSession ?: return state
        if (s.kind != ActiveKind.PLANNED) return state
        // Confirm принимается из BlockEndedSheet (классический поток «Завершил») или
        // из CompletionSheet, открытого через OpenCompletionSheet (W1).
        val sheetBlockId = when (val sheet = state.sheet) {
            is SheetState.BlockEndedSheet -> sheet.blockId
            is SheetState.CompletionSheet -> sheet.blockId
            else -> return state
        }
        if (sheetBlockId != s.blockId) return state
        val block = s.blockId?.let { state.blocks.firstOrNull { b -> b.blockId == it } } ?: return state
        val session = WorkSession(
            sessionId = sessionIdFor(s.blockId, null, null, s.startedAtMs),
            taskId = s.taskId,
            blockId = s.blockId,
            parentBlockId = s.parentBlockId,
            kind = SessionKind.PLANNED_COMPLETED,
            label = null,
            startedAtMs = s.startedAtMs,
            endedAtMs = now,
            durationSec = elapsedMs(s, now) / 1000,
            ease = ease,
        )
        val blocks = state.blocks.map { if (it.blockId == block.blockId) it.copy(progressPercent = 100) else it }
        return state.copy(
            blocks = blocks,
            factSessions = state.factSessions + session,
            activeSession = null,
            sheet = null,
        )
    }

    private fun completeFromTaskDetails(state: PlannerState, blockId: String): PlannerState {
        val s = state.activeSession
        if (s != null && s.kind == ActiveKind.PLANNED && s.blockId == blockId) return state
        val block = state.blocks.firstOrNull { it.blockId == blockId } ?: return state
        val blocks = state.blocks.map { if (it.blockId == blockId) it.copy(progressPercent = 100) else it }
        return state.copy(blocks = blocks, sheet = null)
    }

    private fun completeFromTopThree(state: PlannerState, blockId: String): PlannerState {
        val s = state.activeSession
        if (s != null && s.kind == ActiveKind.PLANNED && s.blockId == blockId) return state
        val block = state.blocks.firstOrNull { it.blockId == blockId } ?: return state
        val blocks = state.blocks.map { if (it.blockId == blockId) it.copy(progressPercent = 100) else it }
        return state.copy(blocks = blocks)
    }

    // ── «Делаю другое» (раздел 7.8) ──

    private fun doElseConfirmed(
        state: PlannerState,
        selection: DoElseSelection,
        handling: DoElseHandling,
        now: Long,
        today: LocalDate,
    ): PlannerState {
        val sheet = state.sheet as? SheetState.DoElseSheet ?: return state
        val originalBlockId = sheet.blockId
        return when (handling) {
            DoElseHandling.LEAVE -> {
                var next = state
                val s = state.activeSession
                if (s != null && s.kind == ActiveKind.PLANNED && s.blockId == originalBlockId) {
                    next = next.copy(factSessions = next.factSessions + interruptedSession(s, now))
                }
                startAlternativeSession(next, originalBlockId, selection, now)
                    .copy(sheet = null, dayPhase = DayPhase.RUNNING)
            }
            DoElseHandling.SHIFT -> state.copy(
                sheet = SheetState.ScheduleSheet(
                    ScheduleRequest(
                        mode = ScheduleMode.RESCHEDULE,
                        blockId = originalBlockId,
                        sourceDate = today,
                        initialDate = today,
                        lockDate = true,
                        purpose = SchedulePurpose.DO_ELSE,
                    ),
                ),
                pendingDoElse = PendingDoElse(originalBlockId, selection),
            )
            DoElseHandling.MOVE -> state.copy(
                sheet = SheetState.ScheduleSheet(
                    ScheduleRequest(
                        mode = ScheduleMode.RESCHEDULE,
                        blockId = originalBlockId,
                        sourceDate = today,
                        initialDate = today.plusDays(1),
                        lockDate = false,
                        purpose = SchedulePurpose.DO_ELSE,
                    ),
                ),
                pendingDoElse = PendingDoElse(originalBlockId, selection),
            )
        }
    }

    private fun startAlternativeSession(
        state: PlannerState,
        originalBlockId: String,
        selection: DoElseSelection,
        now: Long,
    ): PlannerState {
        val session = if (selection.kind == DoElseKind.OTHER_TASK && selection.targetBlockId != null) {
            val target = state.blocks.firstOrNull { it.blockId == selection.targetBlockId }
            if (target != null) {
                val title = state.tasks.firstOrNull { it.taskId == target.taskId }?.title ?: selection.label
                ActiveSession(
                    kind = ActiveKind.PLANNED,
                    taskId = target.taskId,
                    blockId = target.blockId,
                    parentBlockId = originalBlockId,
                    label = title,
                    startedAtMs = now,
                    runningSinceMs = now,
                    accumulatedMs = 0,
                    paused = false,
                    plannedStart = target.start,
                    plannedEnd = target.end,
                )
            } else {
                unplannedSession(originalBlockId, selection, now)
            }
        } else {
            unplannedSession(originalBlockId, selection, now)
        }
        return state.copy(activeSession = session)
    }

    private fun unplannedSession(originalBlockId: String, selection: DoElseSelection, now: Long): ActiveSession =
        ActiveSession(
            kind = ActiveKind.UNPLANNED_WORK,
            taskId = null,
            blockId = null,
            parentBlockId = originalBlockId,
            label = selection.label.ifBlank { defaultLabelFor(selection.kind) },
            startedAtMs = now,
            runningSinceMs = now,
            accumulatedMs = 0,
            paused = false,
            plannedStart = null,
            plannedEnd = null,
            sessionKind = when (selection.kind) {
                DoElseKind.UNPLANNED -> SessionKind.UNPLANNED
                DoElseKind.BREAK -> SessionKind.BREAK
                DoElseKind.PERSONAL -> SessionKind.PERSONAL
                DoElseKind.OTHER -> SessionKind.OTHER
                DoElseKind.OTHER_TASK -> SessionKind.UNPLANNED
            },
        )

    private fun defaultLabelFor(kind: DoElseKind): String = when (kind) {
        DoElseKind.UNPLANNED -> "Незапланированная работа"
        DoElseKind.BREAK -> "Перерыв"
        DoElseKind.PERSONAL -> "Личное"
        DoElseKind.OTHER -> "Другое"
        DoElseKind.OTHER_TASK -> "Другое"
    }

    // ── ScheduleSheet (раздел 10) ──

    private fun scheduleConfirmed(
        state: PlannerState,
        request: ScheduleRequest,
        date: LocalDate,
        start: LocalTime,
        end: LocalTime,
        now: Long,
    ): PlannerState {
        val existing = blocksOn(state, date)
        val exclude = if (request.mode == ScheduleMode.RESCHEDULE) request.blockId else null
        if (ScheduleValidator.validate(start, end, existing, exclude) != null) return state

        var next: PlannerState = when (request.mode) {
            ScheduleMode.RESCHEDULE -> {
                val block = request.blockId?.let { state.blocks.firstOrNull { b -> b.blockId == it } } ?: return state
                if (date == request.sourceDate) {
                    val blocks = state.blocks.map { if (it.blockId == block.blockId) it.copy(date = date, start = start, end = end) else it }
                    state.copy(blocks = blocks)
                } else {
                    val blocks = state.blocks.filterNot { it.blockId == block.blockId } + block.copy(date = date, start = start, end = end)
                    state.copy(blocks = blocks)
                }
            }
            ScheduleMode.SCHEDULE -> {
                val task = request.taskId?.let { state.tasks.firstOrNull { t -> t.taskId == it } } ?: return state
                val newBlock = ScheduledBlock(
                    blockId = "s_${task.taskId}_${yyyyMMdd(date)}",
                    taskId = task.taskId,
                    date = date,
                    start = start,
                    end = end,
                    progressPercent = 0,
                    accent = accentFor(task.priority, task.project),
                )
                val tasks = state.tasks.map { if (it.taskId == task.taskId) it.copy(plannedDate = null) else it }
                state.copy(blocks = state.blocks + newBlock, tasks = tasks)
            }
        }

        if (request.purpose == SchedulePurpose.DO_ELSE && state.pendingDoElse != null) {
            val pending = state.pendingDoElse
            val s = next.activeSession
            if (s != null && s.kind == ActiveKind.PLANNED && s.blockId == pending.blockId) {
                next = next.copy(factSessions = next.factSessions + interruptedSession(s, now))
            }
            next = startAlternativeSession(next, pending.blockId, pending.selection, now)
            next = next.copy(pendingDoElse = null, sheet = null)
        } else {
            next = next.copy(sheet = null)
        }
        return next
    }

    private fun taskDetailsReschedule(state: PlannerState, sameDateOnly: Boolean, today: LocalDate): PlannerState {
        val sheet = state.sheet as? SheetState.TaskDetailsSheet ?: return state
        val block = state.blocks.firstOrNull { it.blockId == sheet.blockId } ?: return state
        val request = if (sameDateOnly) {
            ScheduleRequest(
                mode = ScheduleMode.RESCHEDULE,
                blockId = block.blockId,
                sourceDate = block.date,
                initialDate = block.date,
                lockDate = true,
                purpose = SchedulePurpose.PLAIN,
            )
        } else {
            ScheduleRequest(
                mode = ScheduleMode.RESCHEDULE,
                blockId = block.blockId,
                sourceDate = block.date,
                initialDate = if (block.date == today) today.plusDays(1) else block.date,
                lockDate = false,
                purpose = SchedulePurpose.PLAIN,
            )
        }
        return state.copy(sheet = SheetState.ScheduleSheet(request))
    }

    private fun cancelSheet(state: PlannerState): PlannerState {
        val sheet = state.sheet
        // Отмена ScheduleSheet, открытого из End of Day (purpose=END_DAY): возвращаемся
        // в EndDaySheet (W2). Бизнес-данные не мутируются, dayPhase остаётся END_OF_DAY,
        // transient-рабочий процесс (pendingDoElse) сбрасывается.
        if (sheet is SheetState.ScheduleSheet && sheet.request.purpose == SchedulePurpose.END_DAY) {
            return state.copy(sheet = SheetState.EndDaySheet, pendingDoElse = null)
        }
        var next = state.copy(sheet = null)
        if (sheet is SheetState.ScheduleSheet) next = next.copy(pendingDoElse = null)
        if (sheet is SheetState.EndDaySheet) next = next.copy(dayPhase = DayPhase.RUNNING)
        return next
    }

    // ── End of Day (раздел 11) ──

    private fun endDayTaskHandled(
        state: PlannerState,
        blockId: String,
        action: EndDayAction,
        targetDate: LocalDate?,
        today: LocalDate,
        now: Long,
    ): PlannerState {
        val block = state.blocks.firstOrNull { it.blockId == blockId } ?: return state
        if (action == EndDayAction.CHOOSE_TIME) {
            return state.copy(
                sheet = SheetState.ScheduleSheet(
                    ScheduleRequest(
                        mode = ScheduleMode.RESCHEDULE,
                        blockId = blockId,
                        sourceDate = today,
                        initialDate = today.plusDays(1),
                        lockDate = false,
                        purpose = SchedulePurpose.END_DAY,
                    ),
                ),
            )
        }
        val plannedDate = when (action) {
            EndDayAction.KEEP_TOMORROW -> today.plusDays(1)
            EndDayAction.TO_BACKLOG -> null
            EndDayAction.TO_DATE -> targetDate ?: return state
            EndDayAction.CHOOSE_TIME -> error("unreachable")
        }
        var next = upsertTask(removeBlock(state, blockId), block.taskId, plannedDate)
        val s = next.activeSession
        if (s != null && s.kind == ActiveKind.PLANNED && s.blockId == blockId) {
            val session = WorkSession(
                sessionId = sessionIdFor(s.blockId, null, s.label, s.startedAtMs),
                taskId = s.taskId,
                blockId = s.blockId,
                parentBlockId = s.parentBlockId,
                kind = SessionKind.STOPPED_AT_DAY_END,
                label = s.label,
                startedAtMs = s.startedAtMs,
                endedAtMs = now,
                durationSec = elapsedMs(s, now) / 1000,
                ease = null,
            )
            next = next.copy(factSessions = next.factSessions + session, activeSession = null)
        }
        return next
    }

    private fun endDayFinished(state: PlannerState, now: Long): PlannerState {
        var next = state
        val s = state.activeSession
        if (s != null) {
            val session = WorkSession(
                sessionId = sessionIdFor(s.blockId, s.parentBlockId, s.label, s.startedAtMs),
                taskId = s.taskId,
                blockId = s.blockId,
                parentBlockId = s.parentBlockId,
                kind = SessionKind.STOPPED_AT_DAY_END,
                label = s.label,
                startedAtMs = s.startedAtMs,
                endedAtMs = now,
                durationSec = elapsedMs(s, now) / 1000,
                ease = null,
            )
            next = next.copy(factSessions = next.factSessions + session, activeSession = null)
        }
        return next.copy(
            endDayReview = EndDayReview(atMs = now, energy = state.dayEnergy),
            dayPhase = DayPhase.DAY_COMPLETED,
            sheet = null,
        )
    }

    // ── Прочее ──

    private fun notDoneAction(state: PlannerState, action: NotDoneAction, today: LocalDate): PlannerState {
        val sheet = state.sheet as? SheetState.NotDoneSheet ?: return state
        return when (action) {
            NotDoneAction.LEAVE -> state.copy(sheet = null)
            NotDoneAction.MOVE -> state.copy(
                sheet = SheetState.ScheduleSheet(
                    ScheduleRequest(
                        mode = ScheduleMode.RESCHEDULE,
                        blockId = sheet.blockId,
                        sourceDate = today,
                        initialDate = today,
                        lockDate = false,
                        purpose = SchedulePurpose.PLAIN,
                    ),
                ),
            )
            NotDoneAction.TO_BACKLOG -> {
                val block = state.blocks.firstOrNull { it.blockId == sheet.blockId } ?: return state
                upsertTask(removeBlock(state, sheet.blockId), block.taskId, null).copy(sheet = null)
            }
        }
    }

    private fun quickNoteAdded(state: PlannerState, text: String, kind: NoteKind, now: Long): PlannerState {
        val trimmed = text.trim()
        if (trimmed.isEmpty()) return state
        val note = QuickNote(
            noteId = "note_${now}_${state.quickNotesToday.size}",
            text = trimmed,
            kind = kind,
            atMs = now,
            contextLabel = contextLabelFor(state),
        )
        return state.copy(quickNotesToday = (listOf(note) + state.quickNotesToday).take(5))
    }

    private fun contextLabelFor(state: PlannerState): String? {
        val s = state.activeSession ?: return null
        if (s.kind != ActiveKind.PLANNED) return s.label
        val block = s.blockId?.let { state.blocks.firstOrNull { b -> b.blockId == it } } ?: return null
        return state.tasks.firstOrNull { it.taskId == block.taskId }?.title
    }

    private fun postponedReturned(state: PlannerState, taskId: String, today: LocalDate): PlannerState {
        if (state.tasks.none { it.taskId == taskId }) return state
        val tasks = state.tasks.map { if (it.taskId == taskId) it.copy(postponed = false, plannedDate = today) else it }
        return state.copy(tasks = tasks)
    }

    // ── Internal helpers ──

    private fun removeBlock(state: PlannerState, blockId: String): PlannerState =
        state.copy(blocks = state.blocks.filterNot { it.blockId == blockId })

    private fun upsertTask(state: PlannerState, taskId: String, plannedDate: LocalDate?): PlannerState {
        val existing = state.tasks.firstOrNull { it.taskId == taskId } ?: return state
        val tasks = state.tasks.filterNot { it.taskId == taskId } + existing.copy(plannedDate = plannedDate, postponed = false)
        return state.copy(tasks = tasks)
    }

    private fun interruptedSession(s: ActiveSession, now: Long): WorkSession =
        WorkSession(
            sessionId = sessionIdFor(s.blockId, null, null, s.startedAtMs),
            taskId = s.taskId,
            blockId = s.blockId,
            parentBlockId = s.parentBlockId,
            kind = SessionKind.PLANNED_INTERRUPTED,
            label = null,
            startedAtMs = s.startedAtMs,
            endedAtMs = now,
            durationSec = elapsedMs(s, now) / 1000,
            ease = null,
        )

    private fun sessionIdFor(blockId: String?, parentBlockId: String?, label: String?, startedAtMs: Long): String {
        val anchor = blockId ?: parentBlockId ?: label?.take(8) ?: "work"
        return "ws_${anchor}_$startedAtMs"
    }
}

// ── Производные функции (раздел 4.3 ТЗ) ──

/** Блоки указанной даты. */
fun blocksOn(state: PlannerState, date: LocalDate): List<ScheduledBlock> =
    state.blocks.filter { it.date == date }

/** Date-only задачи даты: plannedDate==date && нет блока с taskId. */
fun dateOnlyTasksOn(state: PlannerState, date: LocalDate): List<Task> =
    state.tasks.filter { t ->
        t.plannedDate == date && state.blocks.none { it.taskId == t.taskId }
    }

/** Группы бэклога (инвариант 3.2: исключаются задачи, имеющие блок). */
fun backlogGroups(state: PlannerState): BacklogGroups {
    val hasBlock: (Task) -> Boolean = { t -> state.blocks.any { it.taskId == t.taskId } }
    return BacklogGroups(
        noDate = state.tasks.filter { it.plannedDate == null && !it.postponed && !hasBlock(it) },
        hasDate = state.tasks.filter { it.plannedDate != null && !it.postponed && !hasBlock(it) },
        postponed = state.tasks.filter { it.postponed && !hasBlock(it) },
    )
}

/** Нагрузка даты: planned + без-времени, overload = totalMin > 480. */
fun workload(state: PlannerState, date: LocalDate): Workload {
    val plannedMin = blocksOn(state, date).sumOf { computeBlockDuration(it) }
    val withoutTimeMin = dateOnlyTasksOn(state, date).sumOf { it.durationMin }
    val totalMin = plannedMin + withoutTimeMin
    return Workload(
        plannedMin = plannedMin,
        withoutTimeMin = withoutTimeMin,
        totalMin = totalMin,
        overload = totalMin > 480,
    )
}

/** HeroState по 5 правилам (раздел 3.8). */
fun deriveHeroState(state: PlannerState, nowMs: Long): HeroState {
    val session = state.activeSession
    if (session != null) {
        return when {
            session.paused -> HeroState.PAUSED
            state.sheet is SheetState.BlockEndedSheet -> HeroState.BLOCK_ENDED_UNRESOLVED
            else -> HeroState.ACTIVE
        }
    }
    return when (state.dayPhase) {
        DayPhase.DAY_NOT_STARTED -> HeroState.DAY_NOT_STARTED
        DayPhase.END_OF_DAY -> HeroState.END_OF_DAY
        DayPhase.DAY_COMPLETED -> HeroState.DAY_COMPLETED
        DayPhase.RUNNING -> {
            val now = Instant.ofEpochMilli(nowMs).atZone(ZoneId.systemDefault()).toLocalTime()
            val unfinished = blocksOn(state, state.todayDate).filter { it.progressPercent < 100 }
            when {
                unfinished.any { !it.start.isAfter(now) && !it.end.isBefore(now) } -> HeroState.READY
                unfinished.any { it.start.isAfter(now) } -> HeroState.UPCOMING
                else -> HeroState.BETWEEN_BLOCKS
            }
        }
    }
}

/** todayTop → блоки сегодняшнего дня; отсутствующие отбрасываются. */
fun topThreeItems(state: PlannerState): List<ScheduledBlock> =
    state.todayTop.mapNotNull { tid ->
        state.blocks.firstOrNull { it.taskId == tid && it.date == state.todayDate }
    }

/** KPI Today по сегодняшним блокам. */
fun kpi(state: PlannerState): Kpi {
    val todaysBlocks = blocksOn(state, state.todayDate)
    val totalToday = todaysBlocks.size
    val doneToday = todaysBlocks.count { it.progressPercent == 100 }
    val topTotal = state.todayTop.size
    val topDone = state.todayTop.count { tid ->
        todaysBlocks.firstOrNull { it.taskId == tid }?.progressPercent == 100
    }
    val blockMin = todaysBlocks.sumOf { computeBlockDuration(it) }
    val doneMin = todaysBlocks.filter { it.progressPercent == 100 }.sumOf { computeBlockDuration(it) }
    return Kpi(
        doneToday = doneToday,
        totalToday = totalToday,
        topDone = topDone,
        topTotal = topTotal,
        doneMin = doneMin,
        blockMin = blockMin,
    )
}

/** Единственная формула elapsed (раздел 4.5). */
fun elapsedMs(session: ActiveSession, nowMs: Long): Long =
    session.accumulatedMs + if (session.paused || session.runningSinceMs == null) 0L else (nowMs - session.runningSinceMs)

/** Цвет блока (раздел 4.4). */
fun accentFor(priority: Priority, project: String): BlockAccent = when {
    project == "Обучение" || project == "Self Development" -> BlockAccent.PURPLE
    priority == Priority.HIGH -> BlockAccent.BLUE
    priority == Priority.LOW -> BlockAccent.ORANGE
    else -> BlockAccent.BLUE
}

/** Agent Today (раздел 7.7): перегруз → низкая энергия → null. */
fun todayAgentInsight(state: PlannerState, nowMs: Long): AgentInsight? {
    if (plannedTodayMin(state) > 480) return AgentInsight(AgentInsightKind.TODAY_OVERLOAD)
    if (state.dayCheckIn?.energy == 0) {
        val firstUnfinished = blocksOn(state, state.todayDate)
            .filter { it.progressPercent < 100 }
            .minByOrNull { it.start }
        if (firstUnfinished != null) {
            return AgentInsight(AgentInsightKind.TODAY_LOW_ENERGY, taskTitle(state, firstUnfinished.taskId))
        }
    }
    return null
}

/** Agent Plan (раздел 6.7): перегруз → просроченный блок → null. */
fun planAgentInsight(state: PlannerState, nowMs: Long): AgentInsight? {
    if (plannedTodayMin(state) > 480) return AgentInsight(AgentInsightKind.PLAN_OVERLOAD)
    val now = Instant.ofEpochMilli(nowMs).atZone(ZoneId.systemDefault()).toLocalTime()
    val firstPastDue = blocksOn(state, state.todayDate)
        .filter { it.progressPercent < 100 && it.end.isBefore(now) }
        .minByOrNull { it.start }
    return firstPastDue?.let { AgentInsight(AgentInsightKind.PLAN_PAST_DUE, taskTitle(state, it.taskId)) }
}

private fun plannedTodayMin(state: PlannerState): Int =
    blocksOn(state, state.todayDate).sumOf { computeBlockDuration(it) }

private fun taskTitle(state: PlannerState, taskId: String): String? =
    state.tasks.firstOrNull { it.taskId == taskId }?.title
