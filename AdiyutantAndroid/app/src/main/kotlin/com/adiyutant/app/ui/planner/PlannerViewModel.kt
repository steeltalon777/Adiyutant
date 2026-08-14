package com.adiyutant.app.ui.planner

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.adiyutant.app.data.PlannerRepository
import com.adiyutant.app.di.ServiceLocator
import com.adiyutant.app.domain.PlannerCore
import com.adiyutant.app.domain.model.PlannerEvent
import com.adiyutant.app.domain.model.PlannerState
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Единый state holder для Today и Plan (R2, ADR-0004).
 *
 * Тонкая обёртка: все переходы делегируются чистому [PlannerCore.reduce];
 * начальное состояние и сохранение — через [PlannerRepository] (будущий шов
 * под Rust Core). Создаётся один раз в [com.adiyutant.app.ui.shell.AppShell]
 * (Activity scope), поэтому переключение вкладок не сбрасывает state и
 * активную сессию.
 */
class PlannerViewModel(
    private val core: PlannerCore,
    private val repository: PlannerRepository,
) : ViewModel() {

    private val _state: MutableStateFlow<PlannerState> = MutableStateFlow(repository.load())
    val state: StateFlow<PlannerState> = _state.asStateFlow()

    fun dispatch(event: PlannerEvent) {
        val next = core.reduce(_state.value, event)
        repository.save(next)
        _state.value = next
    }

    companion object {
        /** Factory из [ServiceLocator] — без аргументов, для [androidx.lifecycle.viewmodel.compose.viewModel]. */
        fun factory(): ViewModelProvider.Factory =
            object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T =
                    PlannerViewModel(
                        core = ServiceLocator.plannerCore,
                        repository = ServiceLocator.plannerRepository,
                    ) as T
            }
    }
}
