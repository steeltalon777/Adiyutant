package com.adiyutant.app.ui.feature.today

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.adiyutant.app.R
import com.adiyutant.app.di.ServiceLocator
import com.adiyutant.app.ui.theme.Muted
import com.adiyutant.app.ui.theme.Accent
import java.time.format.DateTimeFormatter

/**
 * "Сегодня" tab. Currently a shell stub that proves the CorePort → ViewModel
 * → UI data flow with fake data. The full screen from the design spec
 * (docs/design/today-screen.html) is implemented in a later phase.
 */
@Composable
fun TodayScreen() {
    val factory = TodayViewModel.factory(ServiceLocator.corePort)
    val viewModel: TodayViewModel = viewModel(factory = factory)
    val uiState by viewModel.uiState.collectAsState()

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

        when {
            uiState.loading -> {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.Center,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    CircularProgressIndicator(color = Accent)
                }
            }

            uiState.today != null -> {
                val today = requireNotNull(uiState.today)
                Text(
                    text = today.date.format(DateTimeFormatter.ofPattern("EEEE, d MMMM")),
                    style = MaterialTheme.typography.bodyMedium,
                    color = Muted,
                )
                Spacer(Modifier.height(16.dp))
                TodayStubSummary(uiState)
            }

            else -> {
                Text(
                    text = stringResource(R.string.stub_placeholder),
                    style = MaterialTheme.typography.bodyMedium,
                    color = Muted,
                )
            }
        }
    }
}

@Composable
private fun TodayStubSummary(uiState: TodayUiState) {
    val today = uiState.today ?: return
    val activity = uiState.activity

    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text("mode: ${today.mode}", style = MaterialTheme.typography.bodyMedium)
        Text(
            "energy: ${today.energy} · mood: ${today.mood} · check-ins: ${today.checkInCount}",
            style = MaterialTheme.typography.bodyMedium,
            color = Muted,
        )
        Text(
            "habits: ${today.habitEventsDone}/${today.habitCount} · plan: ${today.planItemCount} items",
            style = MaterialTheme.typography.bodyMedium,
            color = Muted,
        )
        if (activity != null) {
            Text(
                "→ ${activity.activeTask} [${activity.activityKind}]",
                style = MaterialTheme.typography.bodyMedium,
                color = Accent,
            )
        }
    }
}
