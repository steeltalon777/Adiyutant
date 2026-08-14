package com.adiyutant.app.ui.shell

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.currentBackStackEntryAsState
import com.adiyutant.app.R
import com.adiyutant.app.domain.blocksOn
import com.adiyutant.app.domain.model.ActiveKind
import com.adiyutant.app.domain.model.PlannerEvent
import com.adiyutant.app.domain.model.PlannerState
import com.adiyutant.app.domain.model.SheetState
import com.adiyutant.app.ui.common.ScheduleSheet
import com.adiyutant.app.ui.common.TaskDetailsSheet
import com.adiyutant.app.ui.navigation.AppNavGraph
import com.adiyutant.app.ui.navigation.Destinations
import com.adiyutant.app.ui.planner.PlannerViewModel
import com.adiyutant.app.ui.theme.Bg
import com.adiyutant.app.ui.theme.Border
import com.adiyutant.app.ui.theme.Muted2
import com.adiyutant.app.ui.theme.Surface2

/**
 * Root shell of the app (ADR-0002, TZ раздел 5).
 *
 * Hoists единственный [PlannerViewModel] на Activity scope: Today и Plan
 * получают один и тот же экземпляр, переключение вкладок не сбрасывает
 * состояние и активную сессию. Общие листы ([SheetState]) рендерятся поверх
 * [AppNavGraph] здесь (Phase 3A: ScheduleSheet, TaskDetailsSheet; Today-листы —
 * Phase 4).
 */
@Composable
fun AppShell(
    plannerViewModel: PlannerViewModel = viewModel(factory = PlannerViewModel.factory()),
) {
    val navController = androidx.navigation.compose.rememberNavController()
    val backStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = backStackEntry?.destination?.route

    val navigateToTab: (Destinations) -> Unit = { tab ->
        navController.navigate(tab.route) {
            popUpTo(navController.graph.startDestinationId) {
                saveState = true
            }
            launchSingleTop = true
            restoreState = true
        }
    }

    val state by plannerViewModel.state.collectAsStateWithLifecycle()
    val sheet = state.sheet

    // Системный back закрывает открытый лист без бизнес-мутаций (R5).
    // Для листов на AdiyutantSheet его перехватывает BackHandler самого листа
    // (LIFO); этот остаётся запасным для прочих листов.
    BackHandler(enabled = sheet != null) {
        plannerViewModel.dispatch(PlannerEvent.CloseSheet)
    }

    Scaffold(
        containerColor = Bg,
        bottomBar = {
            BottomNavBar(
                currentRoute = currentRoute.orEmpty(),
                onTabSelected = navigateToTab,
            )
        },
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .background(MaterialTheme.colorScheme.background),
        ) {
            AppNavGraph(
                navController = navController,
                planner = plannerViewModel,
                onNavigate = navigateToTab,
            )
            SheetHost(
                sheet = sheet,
                state = state,
                dispatch = plannerViewModel::dispatch,
                onNavigate = navigateToTab,
            )
        }
    }
}

/**
 * Host общих листов (Phase 3A): рендерит [SheetState.ScheduleSheet] и
 * [SheetState.TaskDetailsSheet]; остальные листы (Today, EOD) приходят
 * в Phase 4 и пока показываются заглушкой.
 */
@Composable
private fun SheetHost(
    sheet: SheetState?,
    state: PlannerState,
    dispatch: (PlannerEvent) -> Unit,
    onNavigate: (Destinations) -> Unit,
) {
    when (sheet) {
        is SheetState.ScheduleSheet -> {
            val request = sheet.request
            ScheduleSheet(
                request = request,
                today = state.todayDate,
                task = request.taskId?.let { id -> state.tasks.firstOrNull { it.taskId == id } },
                block = request.blockId?.let { id -> state.blocks.firstOrNull { it.blockId == id } },
                blocksOnDate = { date -> blocksOn(state, date) },
                onConfirm = { date, start, end ->
                    dispatch(PlannerEvent.ScheduleConfirmed(request, date, start, end))
                },
                onClose = { dispatch(PlannerEvent.ScheduleCancelled) },
            )
        }
        is SheetState.TaskDetailsSheet -> {
            val block = state.blocks.firstOrNull { it.blockId == sheet.blockId }
            val task = block?.let { b -> state.tasks.firstOrNull { it.taskId == b.taskId } }
            if (block == null || task == null) return
            val session = state.activeSession
            TaskDetailsSheet(
                block = block,
                task = task,
                hasActivePlannedSession = session != null &&
                    session.kind == ActiveKind.PLANNED &&
                    session.blockId == sheet.blockId,
                onReschedule = { sameDateOnly ->
                    dispatch(PlannerEvent.TaskDetailsReschedule(sameDateOnly))
                },
                onComplete = { dispatch(PlannerEvent.TaskDetailsComplete(sheet.blockId)) },
                onOpenToday = { onNavigate(Destinations.TODAY) },
                onClose = { dispatch(PlannerEvent.CloseSheet) },
            )
        }
        else -> {
            if (sheet != null) {
                SheetPlaceholder(onClose = { dispatch(PlannerEvent.CloseSheet) })
            }
        }
    }
}

/** Заглушка для листов Phase 4 (CheckIn, Mode, Progress, DoElse, ...). */
@Composable
private fun SheetPlaceholder(onClose: () -> Unit) {
    Box(modifier = Modifier.fillMaxSize()) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Color.Black.copy(alpha = 0.55f))
                .clickable(onClick = onClose),
        )
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .align(Alignment.BottomCenter)
                .navigationBarsPadding(),
            shape = RoundedCornerShape(topStart = 18.dp, topEnd = 18.dp),
            color = Surface2,
            border = androidx.compose.foundation.BorderStroke(1.dp, Border),
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(24.dp),
            ) {
                Text(
                    text = stringResource(R.string.sheet_host_placeholder),
                    style = MaterialTheme.typography.bodyMedium,
                    color = Muted2,
                )
            }
        }
    }
}
