package com.adiyutant.app.ui.common

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.adiyutant.app.R
import com.adiyutant.app.domain.MONTHS
import com.adiyutant.app.domain.ScheduleValidator
import com.adiyutant.app.domain.computeBlockDuration
import com.adiyutant.app.domain.fmtMins
import com.adiyutant.app.domain.model.ScheduleMode
import com.adiyutant.app.domain.model.ScheduleRequest
import com.adiyutant.app.domain.model.ScheduledBlock
import com.adiyutant.app.domain.model.Task
import com.adiyutant.app.domain.weekdayShort
import com.adiyutant.app.ui.common.ScheduleSheetLogic.ConfirmLabel
import com.adiyutant.app.ui.theme.Accent
import com.adiyutant.app.ui.theme.AccentRed
import com.adiyutant.app.ui.theme.Border
import com.adiyutant.app.ui.theme.Fg
import com.adiyutant.app.ui.theme.Muted
import com.adiyutant.app.ui.theme.Muted2
import com.adiyutant.app.ui.theme.Surface
import com.adiyutant.app.ui.theme.Surface2
import java.time.LocalDate
import java.time.LocalTime

/**
 * Общий ScheduleSheet (contract раздел 10 ТЗ).
 *
 * Формы листа — локальный `rememberSaveable` (R3): VM получает только
 * [onConfirm] с готовыми (date, start, end) и [onClose]. Все переходы —
 * события reducer'а (`ScheduleConfirmed`/`ScheduleCancelled`), бизнес-логика
 * в composable отсутствует; повторная валидация — в reducer.
 *
 * @param task целевая задача (SCHEDULE: date-only/backlog задача; RESCHEDULE: задача блока)
 * @param block сам блок (RESCHEDULE); для SCHEDULE — null
 * @param blocksOnDate проекция блоков даты из state (производная 4.3)
 */
@Composable
fun ScheduleSheet(
    request: ScheduleRequest,
    today: LocalDate,
    task: Task?,
    block: ScheduledBlock?,
    blocksOnDate: (LocalDate) -> List<ScheduledBlock>,
    onConfirm: (date: LocalDate, start: LocalTime, end: LocalTime) -> Unit,
    onClose: () -> Unit,
) {
    val durationMin = block?.let(::computeBlockDuration) ?: task?.durationMin ?: 30
    val title = stringResource(
        if (request.mode == ScheduleMode.SCHEDULE) {
            R.string.sheet_schedule_title_schedule
        } else {
            R.string.sheet_schedule_title_reschedule
        },
    )

    // ── Локальная форма листа (R3) ──
    // Поля — строки «HH:MM», сохраняются при ротации (rememberSaveable).
    // Ключ привязывает форму к identity ScheduleRequest: значения из
    // ScheduleSheet(A) не могут протечь в ScheduleSheet(B) — при новом
    // запросе (mode/taskId/blockId/purpose/initialDate/sourceDate/lockDate)
    // форма пересоздаётся из init (selected date/start/end сбрасываются).
    val formKey = buildString {
        append(request.mode)
        append('|').append(request.taskId)
        append('|').append(request.blockId)
        append('|').append(request.purpose)
        append('|').append(request.initialDate)
        append('|').append(request.sourceDate)
        append('|').append(request.lockDate)
    }
    var dateStr by rememberSaveable(formKey) { mutableStateOf(request.initialDate.toString()) }
    var startStr by rememberSaveable(formKey) { mutableStateOf(block?.start?.toString().orEmpty()) }
    var endStr by rememberSaveable(formKey) { mutableStateOf(block?.end?.toString().orEmpty()) }

    val date = LocalDate.parse(dateStr)
    val start = ScheduleSheetLogic.parseHhMm(startStr)
    val end = ScheduleSheetLogic.parseHhMm(endStr)

    val existing = blocksOnDate(date)
    val excludeBlockId = if (request.mode == ScheduleMode.RESCHEDULE) request.blockId else null
    val inRange = start != null && end != null && ScheduleValidator.isWorkdayTime(start, end)
    val error = if (start == null || end == null) null else ScheduleValidator.validate(start, end, existing, excludeBlockId)
    val canConfirm = start != null && end != null && error == null
    val slots = ScheduleValidator.freeSlots(existing, durationMin, excludeBlockId).take(8)

    val dateLabel: @Composable (LocalDate) -> String = { d ->
        when (ScheduleSheetLogic.dateChoiceKind(d, today)) {
            ScheduleSheetLogic.DateChoiceKind.TODAY -> stringResource(R.string.sheet_today)
            ScheduleSheetLogic.DateChoiceKind.TOMORROW -> stringResource(R.string.sheet_tomorrow)
            ScheduleSheetLogic.DateChoiceKind.OTHER ->
                stringResource(R.string.sheet_date_choice, weekdayShort(d), d.dayOfMonth, MONTHS[d.monthValue - 1])
        }
    }

    fun adjustTime(deltaMin: Int) {
        val (s, e) = if (start == null) {
            ScheduleSheetLogic.initialFromEmpty(durationMin)
        } else {
            ScheduleSheetLogic.shifted(start, durationMin, deltaMin)
        }
        startStr = s.toString()
        endStr = e.toString()
    }

    AdiyutantSheet(title = title, onClose = onClose) {
        Column(modifier = Modifier.padding(horizontal = 20.dp)) {
            if (task != null) {
                // Карточка задачи (10.4): title, project · ~duration.
                Surface(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(bottom = 14.dp),
                    shape = RoundedCornerShape(10.dp),
                    color = Surface,
                    border = BorderStroke(1.dp, Border),
                ) {
                    Column(modifier = Modifier.padding(horizontal = 14.dp, vertical = 12.dp)) {
                        Text(
                            text = task.title,
                            style = MaterialTheme.typography.titleSmall,
                            color = Fg,
                        )
                        Text(
                            text = "${task.project} · ${fmtMins(durationMin)}",
                            style = MaterialTheme.typography.labelSmall,
                            color = Muted2,
                            modifier = Modifier.padding(top = 2.dp),
                        )
                    }
                }
            }

            // Дата: чипы [sourceDate, today, tomorrow, today+2]; lockDate — только
            // текущая дата и неактивные чипы. Смена даты сбрасывает время (10.4).
            Label(text = stringResource(R.string.sheet_date_label))
            val choices = if (request.lockDate) listOf(request.initialDate) else ScheduleSheetLogic.dateChoices(request.sourceDate, today)
            DateChipsRow(
                choices = choices,
                selected = date,
                lockDate = request.lockDate,
                label = dateLabel,
                onSelect = { d ->
                    dateStr = d.toString()
                    startStr = ""
                    endStr = ""
                },
                modifier = Modifier.padding(bottom = 14.dp),
            )

            // Свободные слоты: шаг 30 мин, ≤8 чипов; пусто → «Нет свободных слотов».
            Label(text = stringResource(R.string.sheet_free_slots))
            if (slots.isEmpty()) {
                Text(
                    text = stringResource(R.string.sheet_no_free_slots),
                    style = MaterialTheme.typography.bodySmall,
                    color = Muted,
                    modifier = Modifier.padding(bottom = 14.dp),
                )
            } else {
                FlowRow(
                    modifier = Modifier.padding(bottom = 14.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    slots.forEach { (slotStart, slotEnd) ->
                        Chip(
                            text = "$slotStart–$slotEnd",
                            onClick = {
                                startStr = slotStart.toString()
                                endStr = slotEnd.toString()
                            },
                            selected = start == slotStart && end == slotEnd,
                            tone = ChipTone.ACCENT,
                        )
                    }
                }
            }

            // Время вручную: −1ч, −15, start, «–», end, +15, +1ч (10.4).
            Label(text = stringResource(R.string.sheet_manual_time))
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 10.dp),
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                TimeAdjustButton(
                    text = stringResource(R.string.sheet_time_minus_hour),
                    onClick = { adjustTime(-60) },
                )
                TimeAdjustButton(
                    text = stringResource(R.string.sheet_time_minus15),
                    onClick = { adjustTime(-15) },
                )
                TimeField(
                    value = startStr,
                    placeholder = stringResource(R.string.sheet_start_placeholder),
                    onValueChange = { str ->
                        startStr = str
                        val parsed = ScheduleSheetLogic.parseHhMm(str)
                        endStr = if (parsed != null) ScheduleSheetLogic.endForStart(parsed, durationMin).toString() else ""
                    },
                    modifier = Modifier.weight(1f),
                )
                Text(
                    text = "–",
                    style = MaterialTheme.typography.bodyMedium,
                    color = Muted2,
                )
                TimeField(
                    value = endStr,
                    placeholder = stringResource(R.string.sheet_end_placeholder),
                    onValueChange = { str ->
                        val parsed = ScheduleSheetLogic.parseHhMm(str)
                        if (ScheduleSheetLogic.canSetEnd(start, parsed)) endStr = str
                    },
                    modifier = Modifier.weight(1f),
                )
                TimeAdjustButton(
                    text = stringResource(R.string.sheet_time_plus15),
                    onClick = { adjustTime(15) },
                )
                TimeAdjustButton(
                    text = stringResource(R.string.sheet_time_plus_hour),
                    onClick = { adjustTime(60) },
                )
            }

            // Ошибка валидации под полями (red, 11sp) — 10.3 п.6, точные тексты.
            if (!canConfirm && (startStr.isNotEmpty() || endStr.isNotEmpty())) {
                val message = if (!inRange) {
                    stringResource(R.string.sheet_time_out_of_range)
                } else {
                    stringResource(R.string.sheet_time_overlap)
                }
                Text(
                    text = message,
                    style = MaterialTheme.typography.labelSmall.copy(
                        fontSize = 11.sp,
                        color = AccentRed,
                    ),
                    modifier = Modifier.padding(bottom = 10.dp),
                )
            }

            // Confirm: full-width primary; disabled пока невалидно (10.3 п.8).
            val confirmLabel = when (ScheduleSheetLogic.confirmLabel(request.mode, date, request.sourceDate)) {
                ConfirmLabel.SCHEDULE -> stringResource(R.string.sheet_confirm_schedule)
                ConfirmLabel.SAVE -> stringResource(R.string.sheet_confirm_save)
                ConfirmLabel.MOVE -> stringResource(R.string.sheet_confirm_move)
            }
            val shape = RoundedCornerShape(10.dp)
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(shape)
                    .background(if (canConfirm) Accent else Surface2)
                    .then(if (canConfirm) Modifier.clickable { onConfirm(date, start!!, end!!) } else Modifier)
                    .padding(vertical = 12.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = confirmLabel,
                    style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
                    color = if (canConfirm) Color.White else Muted2,
                )
            }
        }
    }
}

/** Ряд чипов дат с wrap; при [lockDate] чипы неактивны (10.4). */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun DateChipsRow(
    choices: List<LocalDate>,
    selected: LocalDate,
    lockDate: Boolean,
    label: @Composable (LocalDate) -> String,
    onSelect: (LocalDate) -> Unit,
    modifier: Modifier = Modifier,
) {
    FlowRow(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        choices.forEach { d ->
            Chip(
                text = label(d),
                onClick = { onSelect(d) },
                selected = d == selected,
                enabled = !lockDate,
            )
        }
    }
}

/** Кнопка шага времени (−1ч/−15/+15/+1ч): 34×34, Surface2, Border, Muted. */
@Composable
private fun TimeAdjustButton(text: String, onClick: () -> Unit) {
    val shape = RoundedCornerShape(8.dp)
    Box(
        modifier = Modifier
            .width(34.dp)
            .height(34.dp)
            .clip(shape)
            .background(Surface2)
            .border(1.dp, Border, shape)
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.labelSmall,
            color = Muted,
            textAlign = TextAlign.Center,
        )
    }
}

/** Поле «HH:MM»: Surface2 + Border, центр, placeholder Muted2. */
@Composable
private fun TimeField(
    value: String,
    placeholder: String,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val shape = RoundedCornerShape(8.dp)
    Box(
        modifier = modifier
            .clip(shape)
            .background(Surface2)
            .border(1.dp, Border, shape)
            .padding(horizontal = 4.dp, vertical = 6.dp),
        contentAlignment = Alignment.Center,
    ) {
        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            singleLine = true,
            textStyle = TextStyle(
                color = Fg,
                fontSize = 13.sp,
                textAlign = TextAlign.Center,
            ),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
            decorationBox = { innerTextField ->
                if (value.isEmpty()) {
                    Text(
                        text = placeholder,
                        style = MaterialTheme.typography.bodyMedium,
                        color = Muted2,
                    )
                }
                innerTextField()
            },
            modifier = Modifier.fillMaxWidth(),
        )
    }
}
