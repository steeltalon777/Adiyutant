package com.adiyutant.app.domain.model

/** Сохраняемое состояние дня (раздел 3.8 ТЗ). */
enum class DayPhase {
    DAY_NOT_STARTED,
    RUNNING,
    END_OF_DAY,
    DAY_COMPLETED,
}

/** Канонические состояния hero-карточки (раздел 3.8 ТЗ, MEMORY.md). */
enum class HeroState {
    DAY_NOT_STARTED,
    UPCOMING,
    READY,
    ACTIVE,
    PAUSED,
    BLOCK_ENDED_UNRESOLVED,
    BETWEEN_BLOCKS,
    END_OF_DAY,
    DAY_COMPLETED,
}

/** Режим дня: Фокус / Обычный / Лёгкий / Восстановление. */
enum class DayMode {
    FOCUS,
    NORMAL,
    LIGHT,
    RECOVERY,
}

/** Субъективная лёгкость выполнения блока: Легко / Нормально / Тяжело. */
enum class Ease {
    EASY,
    NORMAL,
    HARD,
}

/** Самочувствие в чек-ине: Плохо / Нормально / Хорошо. */
enum class Mood {
    BAD,
    OK,
    GOOD,
}

/** Тип быстрой заметки. */
enum class NoteKind {
    NOTE,
    IDEA,
    PROBLEM,
    SUMMARY,
}

enum class Priority {
    HIGH,
    MEDIUM,
    LOW,
}

/** Цвет-акцент блока в timeline. */
enum class BlockAccent {
    BLUE,
    PURPLE,
    GREEN,
    ORANGE,
    NONE,
}

/** Режим экрана План: День / Неделя / Бэклог. */
enum class PlanMode {
    DAY,
    WEEK,
    BACKLOG,
}

/** Режим ScheduleSheet: создать размещение / перенести существующее. */
enum class ScheduleMode {
    SCHEDULE,
    RESCHEDULE,
}

/** Назначение вызова ScheduleSheet. */
enum class SchedulePurpose {
    PLAIN,
    DO_ELSE,
    END_DAY,
}

/** Тип активной сессии. */
enum class ActiveKind {
    PLANNED,
    UNPLANNED_WORK,
}

/** Тип завершённого факта (WorkSession). */
enum class SessionKind {
    PLANNED_COMPLETED,
    PLANNED_INTERRUPTED,
    UNPLANNED,
    BREAK,
    PERSONAL,
    OTHER,
    STOPPED_AT_DAY_END,
}

/** Выбор «Делаю другое». */
enum class DoElseKind {
    OTHER_TASK,
    UNPLANNED,
    BREAK,
    PERSONAL,
    OTHER,
}

/** Что сделать с исходным блоком при «Делаю другое». */
enum class DoElseHandling {
    LEAVE,
    SHIFT,
    MOVE,
}

/** Судьба блока в листе «Не делал». */
enum class NotDoneAction {
    MOVE,
    TO_BACKLOG,
    LEAVE,
}

/** Действие End of Day для незакрытой задачи. */
enum class EndDayAction {
    KEEP_TOMORROW,
    TO_BACKLOG,
    TO_DATE,
    CHOOSE_TIME,
}

/** Вид инсайта агента (без текстов — тексты живут в strings.xml). */
enum class AgentInsightKind {
    TODAY_OVERLOAD,
    TODAY_LOW_ENERGY,
    PLAN_OVERLOAD,
    PLAN_PAST_DUE,
}
