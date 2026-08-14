package com.adiyutant.app.ui.feature.plan

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.adiyutant.app.R
import com.adiyutant.app.ui.navigation.Destinations
import com.adiyutant.app.ui.planner.PlannerViewModel
import com.adiyutant.app.ui.theme.Muted
import com.adiyutant.app.ui.theme.Muted2

/**
 * "План" tab (TZ раздел 6).
 *
 * Phase 2: временная заглушка на общем [PlannerViewModel]. Полная реализация
 * (День/Неделя/Бэклог, ScheduleSheet-точки входа, Plan Agent) — Phase 3B.
 */
@Composable
fun PlanScreen(
    planner: PlannerViewModel,
    onNavigate: (Destinations) -> Unit,
) {
    val state by planner.state.collectAsStateWithLifecycle()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
    ) {
        Text(
            text = stringResource(R.string.tab_plan),
            style = MaterialTheme.typography.displayLarge,
            color = MaterialTheme.colorScheme.onBackground,
        )
        Spacer(Modifier.height(8.dp))
        Text(
            text = stringResource(R.string.plan_screen_placeholder),
            style = MaterialTheme.typography.bodyMedium,
            color = Muted,
        )
        Spacer(Modifier.height(16.dp))
        // Временная диагностика общего state (заменится в Phase 3B).
        Text(
            text = "mode: ${state.planUi.mode} · selectedDate: ${state.planUi.selectedDate}",
            style = MaterialTheme.typography.bodyMedium,
            color = Muted2,
        )
    }
}
