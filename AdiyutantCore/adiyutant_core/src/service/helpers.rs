use crate::dto::{self, PlanItemDto};
use crate::error::CoreError;
use crate::id::Id;
use crate::model::day_plan::PlanItem;

pub(crate) fn plan_item_to_dto(item: &PlanItem) -> PlanItemDto {
    PlanItemDto {
        id: item.id.value().to_string(),
        title: item.title.clone(),
        description: dto::option_string(&item.description),
        quadrant: item.quadrant.as_str().to_string(),
        planned_start: item
            .planned_start
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_default(),
        planned_end: item
            .planned_end
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_default(),
        status: item.status.as_str().to_string(),
        priority: item.priority,
        source: item.source.as_str().to_string(),
        created_at: item.created_at.inner().to_rfc3339(),
        updated_at: item.updated_at.inner().to_rfc3339(),
    }
}

pub(crate) fn parse_item_id(s: &str) -> Result<Id<PlanItem>, CoreError> {
    let uuid = uuid::Uuid::parse_str(s)
        .map_err(|e| CoreError::InvalidInput(format!("invalid item id: {e}")))?;
    Ok(Id::from_uuid(uuid))
}

pub(crate) fn parse_id<T>(s: &str) -> Result<Id<T>, CoreError> {
    let uuid = uuid::Uuid::parse_str(s)
        .map_err(|e| CoreError::InvalidInput(format!("invalid id: {e}")))?;
    Ok(Id::from_uuid(uuid))
}
