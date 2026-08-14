package com.adiyutant.app.domain.model

import java.time.LocalDate
import java.time.LocalTime
import com.adiyutant.app.domain.model.NotDoneAction as NotDoneActionEnum

/**
 * Каталог событий (раздел 4.8 ТЗ). Единственный способ изменить [PlannerState].
 */
sealed interface PlannerEvent {
    data class CheckInSubmitted(val energy: Int, val focus: Int, val mood: Mood, val obstacle: String?) : PlannerEvent
    data class DayModeSelected(val mode: DayMode) : PlannerEvent
    data class StartBlock(val blockId: String) : PlannerEvent
    data object PauseSession : PlannerEvent
    data object ResumeSession : PlannerEvent
    data object FinishActiveSession : PlannerEvent // «Завершил» / «Завершить»
    data object BlockEndedStillWorking : PlannerEvent // «Ещё работаю»
    data object BlockEndedNotDone : PlannerEvent // «Не делал»
    data class OpenCompletionSheet(val blockId: String) : PlannerEvent // открыть CompletionSheet для блока
    data class CompletionConfirmed(val ease: Ease) : PlannerEvent
    data class NotDoneAction(val action: NotDoneActionEnum) : PlannerEvent // MOVE | TO_BACKLOG | LEAVE
    data class DoElseRequested(val blockId: String) : PlannerEvent
    data class DoElseConfirmed(val selection: DoElseSelection, val handling: DoElseHandling) : PlannerEvent
    data class OpenScheduleSheet(val request: ScheduleRequest) : PlannerEvent
    data class ScheduleConfirmed(val request: ScheduleRequest, val date: LocalDate, val start: LocalTime, val end: LocalTime) : PlannerEvent
    data object ScheduleCancelled : PlannerEvent
    data class OpenTaskDetails(val blockId: String, val sourceDate: LocalDate) : PlannerEvent
    data class TaskDetailsReschedule(val sameDateOnly: Boolean) : PlannerEvent // Изменить время / Перенести
    data class TaskDetailsComplete(val blockId: String) : PlannerEvent
    data object CloseSheet : PlannerEvent
    data class TopThreeStart(val blockId: String) : PlannerEvent
    data class TopThreeComplete(val blockId: String) : PlannerEvent
    data class QuickNoteAdded(val text: String, val kind: NoteKind) : PlannerEvent
    data object OpenEndOfDay : PlannerEvent
    data class EndDayTaskHandled(val blockId: String, val action: EndDayAction, val targetDate: LocalDate?) : PlannerEvent
    data class EndDayEnergySelected(val energy: Int) : PlannerEvent
    data object EndDayFinished : PlannerEvent
    data class PlanModeSelected(val mode: PlanMode) : PlannerEvent
    data class PlanDateSelected(val date: LocalDate) : PlannerEvent
    data class ViewMonthShifted(val deltaMonths: Int) : PlannerEvent
    data class PostponedReturned(val taskId: String) : PlannerEvent
}

/** Выбор «Делаю другое» (раздел 4.8 ТЗ). */
data class DoElseSelection(
    val kind: DoElseKind,
    val label: String,
    val targetBlockId: String?,
)
