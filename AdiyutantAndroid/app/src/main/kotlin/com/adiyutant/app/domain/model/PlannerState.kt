package com.adiyutant.app.domain.model

import java.time.LocalDate
import java.time.YearMonth

/**
 * Единый immutable state (раздел 4.2 ТЗ). Today и Plan рендерятся из одного
 * экземпляра; изменения — только через события [com.adiyutant.app.domain.PlannerEvent].
 */
data class PlannerState(
    val todayDate: LocalDate,                 // зафиксирован при старте
    val tasks: List<Task>,                    // единый каталог
    val blocks: List<ScheduledBlock>,         // все размещения
    val activeSession: ActiveSession?,        // переживает переключение вкладок
    val factSessions: List<WorkSession>,      // завершённые факт-сессии
    val dayCheckIn: CheckIn?,
    val dayPhase: DayPhase,
    val dayMode: DayMode,
    val dayEnergy: Int,                       // 0..3, последний чек-ин
    val todayTop: List<String>,               // taskId, максимум 3
    val quickNotesToday: List<QuickNote>,     // максимум 5
    val endDayReview: EndDayReview?,
    val planUi: PlanUiState,                  // переживает вкладки
    val sheet: SheetState?,
    val pendingDoElse: PendingDoElse?,        // двухшаговый «Делаю другое»
)

data class PlanUiState(
    val mode: PlanMode = PlanMode.WEEK,
    val selectedDate: LocalDate,
    val viewMonth: YearMonth,
)

/** Единственный активный лист VM-уровня. */
sealed interface SheetState {
    data object CheckInSheet : SheetState
    data object ModeSheet : SheetState
    data object ProgressSheet : SheetState
    data class DoElseSheet(val blockId: String) : SheetState
    data class BlockEndedSheet(val blockId: String) : SheetState
    data class NotDoneSheet(val blockId: String) : SheetState
    data class CompletionSheet(val blockId: String) : SheetState
    data object QuickNoteSheet : SheetState
    data object EndDaySheet : SheetState
    data class ScheduleSheet(val request: ScheduleRequest) : SheetState
    data class TaskDetailsSheet(val blockId: String, val sourceDate: LocalDate) : SheetState
}

/** Контракт ScheduleSheet (раздел 10.1 ТЗ). */
data class ScheduleRequest(
    val mode: ScheduleMode,
    val taskId: String? = null,     // SCHEDULE
    val blockId: String? = null,    // RESCHEDULE
    val sourceDate: LocalDate,
    val initialDate: LocalDate,
    val lockDate: Boolean,
    val purpose: SchedulePurpose,
)

/** Двухшаговый «Делаю другое» → ждёт подтверждения ScheduleSheet. */
data class PendingDoElse(val blockId: String, val selection: DoElseSelection)

data class EndDayReview(val atMs: Long, val energy: Int)

// Производные данные (раздел 4.3 ТЗ) — чистые, без UI-текстов.

data class Workload(
    val plannedMin: Int,
    val withoutTimeMin: Int,
    val totalMin: Int,
    val overload: Boolean, // totalMin > 480
)

data class Kpi(
    val doneToday: Int,
    val totalToday: Int,
    val topDone: Int,
    val topTotal: Int,
    val doneMin: Int,
    val blockMin: Int,
)

data class BacklogGroups(
    val noDate: List<Task>,
    val hasDate: List<Task>,
    val postponed: List<Task>,
)

/** Инсайт агента — чистые данные без текстов. */
data class AgentInsight(
    val kind: AgentInsightKind,
    val blockTitle: String? = null,
)
