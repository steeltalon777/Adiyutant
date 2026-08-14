package com.adiyutant.app.data

import com.adiyutant.app.domain.model.PlannerState

/** Хранит [PlannerState] в памяти; перезапуск приложения возвращает seed. */
class InMemoryPlannerRepository(initial: PlannerState) : PlannerRepository {

    private var state: PlannerState = initial

    override fun load(): PlannerState = state

    override fun save(state: PlannerState) {
        this.state = state
    }
}
