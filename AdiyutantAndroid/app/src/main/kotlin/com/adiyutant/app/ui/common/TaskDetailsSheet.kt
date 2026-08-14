package com.adiyutant.app.ui.common

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.adiyutant.app.R
import com.adiyutant.app.domain.model.ScheduledBlock
import com.adiyutant.app.domain.model.Task
import com.adiyutant.app.ui.theme.Accent
import com.adiyutant.app.ui.theme.AccentGreen
import com.adiyutant.app.ui.theme.Bg
import com.adiyutant.app.ui.theme.Border
import com.adiyutant.app.ui.theme.Fg
import com.adiyutant.app.ui.theme.Muted
import com.adiyutant.app.ui.theme.Surface
import com.adiyutant.app.ui.theme.Surface2

/**
 * TaskDetailsSheet (App.jsx 1977–2008, R11).
 *
 * «Изменить время» / «Перенести» → [onReschedule] (reducer строит
 * ScheduleRequest сам); «Завершить» → [onComplete]. Если на блоке активна
 * PLANNED-сессия — «Завершить» скрывается, вместо него CTA «Открыть в
 * Сегодня» → [onOpenToday] (блок завершается только в Today; reducer
 * `TaskDetailsComplete` — no-op в этой ситуации).
 */
@Composable
fun TaskDetailsSheet(
    block: ScheduledBlock,
    task: Task,
    hasActivePlannedSession: Boolean,
    onReschedule: (sameDateOnly: Boolean) -> Unit,
    onComplete: () -> Unit,
    onOpenToday: () -> Unit,
    onClose: () -> Unit,
) {
    AdiyutantSheet(title = stringResource(R.string.sheet_task_title), onClose = onClose) {
        Column(modifier = Modifier.padding(horizontal = 20.dp)) {
            // Карточка: title 15/600; «HH:MM–HH:MM · project» 12 Muted.
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 18.dp),
                shape = RoundedCornerShape(12.dp),
                color = Surface,
                border = BorderStroke(1.dp, Border),
            ) {
                Column(modifier = Modifier.padding(14.dp)) {
                    Text(
                        text = task.title,
                        style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
                        color = Fg,
                    )
                    Text(
                        text = "$block.start–$block.end · ${task.project}",
                        style = MaterialTheme.typography.bodySmall,
                        color = Muted,
                        modifier = Modifier.padding(top = 6.dp),
                    )
                }
            }

            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                SheetActionButton(
                    text = stringResource(R.string.sheet_task_change_time),
                    onClick = { onReschedule(true) },
                    style = SheetActionStyle.ACCENT_OUTLINE,
                )
                SheetActionButton(
                    text = stringResource(R.string.sheet_task_move),
                    onClick = { onReschedule(false) },
                    style = SheetActionStyle.SURFACE,
                )
                if (hasActivePlannedSession) {
                    SheetActionButton(
                        text = stringResource(R.string.sheet_task_open_today),
                        onClick = onOpenToday,
                        style = SheetActionStyle.PRIMARY,
                    )
                } else {
                    SheetActionButton(
                        text = stringResource(R.string.sheet_task_complete),
                        onClick = onComplete,
                        style = SheetActionStyle.GREEN,
                    )
                }
            }
        }
    }
}

private enum class SheetActionStyle { ACCENT_OUTLINE, SURFACE, PRIMARY, GREEN }

@Composable
private fun SheetActionButton(
    text: String,
    onClick: () -> Unit,
    style: SheetActionStyle,
) {
    val shape = RoundedCornerShape(10.dp)
    val (background, border, content) = when (style) {
        SheetActionStyle.ACCENT_OUTLINE -> Triple(
            Accent.copy(alpha = 0.13f),
            Accent.copy(alpha = 0.33f),
            Accent,
        )
        SheetActionStyle.SURFACE -> Triple(Surface2, Border, Fg)
        SheetActionStyle.PRIMARY -> Triple(Accent, Color.Transparent, Color.White)
        SheetActionStyle.GREEN -> Triple(AccentGreen, Color.Transparent, Bg)
    }
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape)
            .background(background)
            .border(1.dp, border, shape)
            .clickable(onClick = onClick)
            .padding(vertical = 12.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.SemiBold),
            color = content,
        )
    }
}
