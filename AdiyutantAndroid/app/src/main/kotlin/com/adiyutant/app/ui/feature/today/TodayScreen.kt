package com.adiyutant.app.ui.feature.today

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
 * "Сегодня" tab (TZ раздел 7).
 *
 * Phase 2: временная заглушка на общем [PlannerViewModel] — экран рендерится
 * из единого state, hoisted на Activity scope. Полная реализация
 * (header/KPI/hero/Agent/Top3/QuickNote + Today-листы) — Phase 4.
 */
@Composable
fun TodayScreen(
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
            text = stringResource(R.string.tab_today),
            style = MaterialTheme.typography.displayLarge,
            color = MaterialTheme.colorScheme.onBackground,
        )
        Spacer(Modifier.height(8.dp))
        Text(
            text = stringResource(R.string.today_screen_placeholder),
            style = MaterialTheme.typography.bodyMedium,
            color = Muted,
        )
        Spacer(Modifier.height(16.dp))
        // Временная диагностика общего state (заменится в Phase 4).
        Text(
            text = "dayPhase: ${state.dayPhase} · today: ${state.todayDate}",
            style = MaterialTheme.typography.bodyMedium,
            color = Muted2,
        )
    }
}
